use crate::aws::media_convert;
use crate::aws::ConfigCredentialsProvider;
use crate::config::Config;
use crate::fairings::sqs::QueueHandler;
use crate::guards::DbConn;
use anyhow::Result;
use hyper_tls::HttpsConnector;
use rusoto_core::HttpClient;
use rusoto_mediaconvert::{
  AacSettings, AudioCodecSettings, AudioDescription, AudioSelector, ContainerSettings,
  CreateJobRequest, DestinationSettings, FileGroupSettings, FrameCaptureSettings, H264Settings,
  Input, JobSettings, MediaConvert, MediaConvertClient, Mp4Settings, Output, OutputGroup,
  OutputGroupSettings, S3DestinationAccessControl, S3DestinationSettings, VideoCodecSettings,
  VideoDescription, VideoSelector,
};
use rusoto_s3::{DeleteObjectRequest, HeadObjectRequest, S3Client, S3};
use rusoto_signature::Region;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RekognitionNotificationMessageVideo {
  s3_object_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RekognitionNotificationMessage {
  video: RekognitionNotificationMessageVideo,
  status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RekognitionNotification {
  message: String,
}

pub struct RekognitionHandler {
  config: Config,
  client: S3Client,
}

#[async_trait]
impl QueueHandler for RekognitionHandler {
  fn name(&self) -> &'static str {
    "rekognition"
  }

  fn new(_db_conn: DbConn, config: Config) -> Self {
    let client = S3Client::new_with(
      HttpClient::from_connector(HttpsConnector::new()),
      ConfigCredentialsProvider::new(config.clone()),
      Region::EuWest2,
    );

    Self { config, client }
  }

  fn url(&self) -> String {
    self.config.rekognition_queue_url.clone()
  }

  async fn instance_id(&self, message: String) -> Result<Option<String>> {
    let notification = self.message_body(message)?;
    let message = serde_json::from_str::<RekognitionNotificationMessage>(&notification.message)?;

    let head_object_params = HeadObjectRequest {
      bucket: self.config.uploads_bucket.clone(),
      key: message.video.s3_object_name.clone(),
      ..Default::default()
    };

    let object = self.client.head_object(head_object_params).await?;

    let instance_id: Option<String> = object
      .metadata
      .and_then(|metadata| metadata.get("instance-id").cloned());

    Ok(instance_id)
  }

  async fn handle(&self, message: String) -> Result<()> {
    let notification = self.message_body(message)?;
    let message = serde_json::from_str::<RekognitionNotificationMessage>(&notification.message)?;

    if message.status != "SUCCEEDED" {
      let delete_object_params = DeleteObjectRequest {
        bucket: self.config.uploads_bucket.clone(),
        key: message.video.s3_object_name.clone(),
        ..Default::default()
      };

      self.client.delete_object(delete_object_params).await?;

      return Ok(());
    }

    let endpoints_response = media_convert::describe_endpoints(self.config.clone()).await?;
    let endpoints = endpoints_response.endpoints;

    let endpoint = endpoints
      .first()
      .ok_or_else(|| anyhow::anyhow!("No endpoint found"))?;

    let media_convert = MediaConvertClient::new_with(
      HttpClient::from_connector(HttpsConnector::new()),
      ConfigCredentialsProvider::new(self.config.clone()),
      Region::Custom {
        name: Region::EuWest2.name().to_owned(),
        endpoint: endpoint.url.clone(),
      },
    );

    media_convert
      .create_job(CreateJobRequest {
        queue: Some(self.config.media_convert_queue_arn.clone()),
        role: self.config.media_convert_role_arn.clone(),
        settings: JobSettings {
          output_groups: Some(vec![OutputGroup {
            name: Some("File Group".to_owned()),
            output_group_settings: Some(OutputGroupSettings {
              type_: Some("FILE_GROUP_SETTINGS".to_owned()),
              file_group_settings: Some(FileGroupSettings {
                destination: Some(format!(
                  "s3://{}/recordings/processed/",
                  self.config.uploads_bucket
                )),
                destination_settings: Some(DestinationSettings {
                  s3_settings: Some(S3DestinationSettings {
                    access_control: Some(S3DestinationAccessControl {
                      canned_acl: Some("PUBLIC_READ".to_owned()),
                    }),
                    ..Default::default()
                  }),
                }),
              }),
              ..Default::default()
            }),
            outputs: Some(vec![
              Output {
                video_description: Some(VideoDescription {
                  scaling_behavior: Some("DEFAULT".to_owned()),
                  timecode_insertion: Some("DISABLED".to_owned()),
                  anti_alias: Some("ENABLED".to_owned()),
                  sharpness: Some(50),
                  codec_settings: Some(VideoCodecSettings {
                    codec: Some("H_264".to_owned()),
                    h264_settings: Some(H264Settings {
                      interlace_mode: Some("PROGRESSIVE".to_owned()),
                      number_reference_frames: Some(3),
                      syntax: Some("DEFAULT".to_owned()),
                      softness: Some(0),
                      gop_closed_cadence: Some(1),
                      gop_size: Some(24.0),
                      slices: Some(1),
                      gop_b_reference: Some("DISABLED".to_owned()),
                      slow_pal: Some("DISABLED".to_owned()),
                      spatial_adaptive_quantization: Some("ENABLED".to_owned()),
                      temporal_adaptive_quantization: Some("ENABLED".to_owned()),
                      flicker_adaptive_quantization: Some("DISABLED".to_owned()),
                      entropy_encoding: Some("CABAC".to_owned()),
                      bitrate: Some(4500000),
                      framerate_control: Some("SPECIFIED".to_owned()),
                      rate_control_mode: Some("CBR".to_owned()),
                      codec_profile: Some("HIGH".to_owned()),
                      telecine: Some("NONE".to_owned()),
                      min_i_interval: Some(0),
                      codec_level: Some("LEVEL_3_1".to_owned()),
                      adaptive_quantization: Some("HIGH".to_owned()),
                      field_encoding: Some("PAFF".to_owned()),
                      scene_change_detect: Some("ENABLED".to_owned()),
                      quality_tuning_level: Some("SINGLE_PASS_HQ".to_owned()),
                      framerate_conversion_algorithm: Some("DUPLICATE_DROP".to_owned()),
                      unregistered_sei_timecode: Some("DISABLED".to_owned()),
                      gop_size_units: Some("FRAMES".to_owned()),
                      par_control: Some("INITIALIZE_FROM_SOURCE".to_owned()),
                      number_b_frames_between_reference_frames: Some(3),
                      repeat_pps: Some("DISABLED".to_owned()),
                      hrd_buffer_size: Some(9000000),
                      hrd_buffer_initial_fill_percentage: Some(90),
                      framerate_numerator: Some(24000),
                      framerate_denominator: Some(1001),
                      ..Default::default()
                    }),
                    ..Default::default()
                  }),
                  afd_signaling: Some("NONE".to_owned()),
                  drop_frame_timecode: Some("ENABLED".to_owned()),
                  respond_to_afd: Some("NONE".to_owned()),
                  color_metadata: Some("INSERT".to_owned()),
                  width: Some(1280),
                  height: Some(720),
                  ..Default::default()
                }),
                audio_descriptions: Some(vec![AudioDescription {
                  audio_type_control: Some("FOLLOW_INPUT".to_owned()),
                  codec_settings: Some(AudioCodecSettings {
                    codec: Some("AAC".to_owned()),
                    aac_settings: Some(AacSettings {
                      audio_description_broadcaster_mix: Some("NORMAL".to_owned()),
                      bitrate: Some(96000),
                      rate_control_mode: Some("CBR".to_owned()),
                      codec_profile: Some("LC".to_owned()),
                      coding_mode: Some("CODING_MODE_2_0".to_owned()),
                      raw_format: Some("NONE".to_owned()),
                      sample_rate: Some(48000),
                      specification: Some("MPEG4".to_owned()),
                      ..Default::default()
                    }),
                    ..Default::default()
                  }),
                  language_code_control: Some("FOLLOW_INPUT".to_owned()),
                  ..Default::default()
                }]),
                container_settings: Some(ContainerSettings {
                  container: Some("MP4".to_owned()),
                  mp_4_settings: Some(Mp4Settings {
                    cslg_atom: Some("INCLUDE".to_owned()),
                    free_space_box: Some("EXCLUDE".to_owned()),
                    moov_placement: Some("PROGRESSIVE_DOWNLOAD".to_owned()),
                    ..Default::default()
                  }),
                  ..Default::default()
                }),
                name_modifier: Some("_720p".to_owned()),
                ..Default::default()
              },
              Output {
                container_settings: Some(ContainerSettings {
                  container: Some("RAW".to_owned()),
                  ..Default::default()
                }),
                extension: Some("jpg".to_owned()),
                name_modifier: Some("_thumbnail".to_owned()),
                video_description: Some(VideoDescription {
                  height: Some(720),
                  scaling_behavior: Some("DEFAULT".to_owned()),
                  timecode_insertion: Some("DISABLED".to_owned()),
                  anti_alias: Some("ENABLED".to_owned()),
                  sharpness: Some(50),
                  codec_settings: Some(VideoCodecSettings {
                    codec: Some("FRAME_CAPTURE".to_owned()),
                    frame_capture_settings: Some(FrameCaptureSettings {
                      framerate_numerator: Some(24),
                      framerate_denominator: Some(240),
                      max_captures: Some(2),
                      quality: Some(80),
                      ..Default::default()
                    }),
                    ..Default::default()
                  }),
                  afd_signaling: Some("NONE".to_owned()),
                  drop_frame_timecode: Some("ENABLED".to_owned()),
                  respond_to_afd: Some("NONE".to_owned()),
                  color_metadata: Some("INSERT".to_owned()),
                  ..Default::default()
                }),
                ..Default::default()
              },
            ]),
            ..Default::default()
          }]),
          ad_avail_offset: Some(0),
          inputs: Some(vec![Input {
            audio_selectors: Some({
              let mut map = std::collections::HashMap::new();
              map.insert(
                "Audio Selector 1".to_owned(),
                AudioSelector {
                  offset: Some(0),
                  default_selection: Some("DEFAULT".to_owned()),
                  selector_type: Some("TRACK".to_owned()),
                  program_selection: Some(1),
                  ..Default::default()
                },
              );
              map
            }),
            video_selector: Some(VideoSelector {
              color_space: Some("FOLLOW".to_owned()),
              ..Default::default()
            }),
            filter_enable: Some("AUTO".to_owned()),
            psi_control: Some("USE_PSI".to_owned()),
            filter_strength: Some(0),
            deblock_filter: Some("DISABLED".to_owned()),
            denoise_filter: Some("DISABLED".to_owned()),
            timecode_source: Some("EMBEDDED".to_owned()),
            file_input: Some(format!(
              "s3://{}/{}",
              self.config.uploads_bucket, message.video.s3_object_name
            )),
            ..Default::default()
          }]),
          ..Default::default()
        },
        ..Default::default()
      })
      .await?;

    Ok(())
  }
}

impl RekognitionHandler {
  fn message_body(&self, message: String) -> Result<RekognitionNotification> {
    Ok(serde_json::from_str(&message)?)
  }
}
