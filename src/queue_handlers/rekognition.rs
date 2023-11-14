use crate::aws;
use crate::aws::media_convert::describe_endpoints;
use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use rusoto_core::HttpClient;
use rusoto_mediaconvert::MediaConvert;
use rusoto_s3::{HeadObjectRequest, S3Client, S3};
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
    let mut builder = hyper::Client::builder();
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    builder.pool_max_idle_per_host(0);

    let client = S3Client::new_with(
      HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
      aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
      Region::EuWest2,
    );

    Self { config, client }
  }

  fn url(&self) -> String {
    self.config.rekognition_queue_url.clone()
  }

  async fn instance_id(&self, message: String) -> Result<Option<String>, QueueHandlerError> {
    let notification = self.message_body(message)?;

    let message = serde_json::from_str::<RekognitionNotificationMessage>(&notification.message)
      .map_err(anyhow::Error::from)?;

    let object = self
      .client
      .head_object(HeadObjectRequest {
        bucket: self.config.uploads_bucket.clone(),
        key: message.video.s3_object_name,
        ..Default::default()
      })
      .await
      .map_err(anyhow::Error::from)?;

    let instance_id: Option<String> = object
      .metadata
      .and_then(|metadata| metadata.get("instance-id").cloned());

    Ok(instance_id)
  }

  async fn handle(&self, message: String) -> Result<(), QueueHandlerError> {
    let notification = self.message_body(message)?;
    let mut builder = hyper::Client::builder();

    builder.pool_max_idle_per_host(0);

    let message = serde_json::from_str::<RekognitionNotificationMessage>(&notification.message)
      .map_err(anyhow::Error::from)?;

    if message.status != "SUCCEEDED" {
      self
        .client
        .delete_object(rusoto_s3::DeleteObjectRequest {
          bucket: self.config.uploads_bucket.to_owned(),
          key: message.video.s3_object_name,
          ..Default::default()
        })
        .await
        .map_err(anyhow::Error::from)?;

      return Ok(());
    }

    let endpoints_response = describe_endpoints(self.config.clone())
      .await
      .map_err(anyhow::Error::from)?;

    let endpoints = endpoints_response.endpoints;

    let endpoint = endpoints
      .first()
      .ok_or_else(|| anyhow::anyhow!("No endpoint found"))?;

    let media_convert = rusoto_mediaconvert::MediaConvertClient::new_with(
      HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
      aws::CredentialsProvider::new(
        self.config.aws_access_key_id.clone(),
        self.config.aws_secret_key.clone(),
      ),
      Region::Custom {
        name: Region::EuWest2.name().to_owned(),
        endpoint: endpoint.url.clone(),
      },
    );

    media_convert
      .create_job(rusoto_mediaconvert::CreateJobRequest {
        queue: Some(self.config.media_convert_queue_arn.clone()),
        role: self.config.media_convert_role_arn.clone(),

        settings: rusoto_mediaconvert::JobSettings {
          output_groups: Some(vec![rusoto_mediaconvert::OutputGroup {
            name: Some("File Group".to_owned()),
            output_group_settings: Some(rusoto_mediaconvert::OutputGroupSettings {
              type_: Some("FILE_GROUP_SETTINGS".to_owned()),
              file_group_settings: Some(rusoto_mediaconvert::FileGroupSettings {
                destination: Some(format!(
                  "s3://{}/recordings/processed/",
                  self.config.uploads_bucket
                )),
                destination_settings: Some(rusoto_mediaconvert::DestinationSettings {
                  s3_settings: Some(rusoto_mediaconvert::S3DestinationSettings {
                    access_control: Some(rusoto_mediaconvert::S3DestinationAccessControl {
                      canned_acl: Some("PUBLIC_READ".to_owned()),
                    }),
                    ..Default::default()
                  }),
                }),
              }),
              ..Default::default()
            }),
            outputs: Some(vec![
              rusoto_mediaconvert::Output {
                video_description: Some(rusoto_mediaconvert::VideoDescription {
                  scaling_behavior: Some("DEFAULT".to_owned()),
                  timecode_insertion: Some("DISABLED".to_owned()),
                  anti_alias: Some("ENABLED".to_owned()),
                  sharpness: Some(50),
                  codec_settings: Some(rusoto_mediaconvert::VideoCodecSettings {
                    codec: Some("H_264".to_owned()),
                    h264_settings: Some(rusoto_mediaconvert::H264Settings {
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
                audio_descriptions: Some(vec![rusoto_mediaconvert::AudioDescription {
                  audio_type_control: Some("FOLLOW_INPUT".to_owned()),
                  codec_settings: Some(rusoto_mediaconvert::AudioCodecSettings {
                    codec: Some("AAC".to_owned()),
                    aac_settings: Some(rusoto_mediaconvert::AacSettings {
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
                container_settings: Some(rusoto_mediaconvert::ContainerSettings {
                  container: Some("MP4".to_owned()),
                  mp_4_settings: Some(rusoto_mediaconvert::Mp4Settings {
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
              rusoto_mediaconvert::Output {
                container_settings: Some(rusoto_mediaconvert::ContainerSettings {
                  container: Some("RAW".to_owned()),
                  ..Default::default()
                }),
                extension: Some("jpg".to_owned()),
                name_modifier: Some("_thumbnail".to_owned()),
                video_description: Some(rusoto_mediaconvert::VideoDescription {
                  height: Some(720),
                  scaling_behavior: Some("DEFAULT".to_owned()),
                  timecode_insertion: Some("DISABLED".to_owned()),
                  anti_alias: Some("ENABLED".to_owned()),
                  sharpness: Some(50),
                  codec_settings: Some(rusoto_mediaconvert::VideoCodecSettings {
                    codec: Some("FRAME_CAPTURE".to_owned()),
                    frame_capture_settings: Some(rusoto_mediaconvert::FrameCaptureSettings {
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
          inputs: Some(vec![rusoto_mediaconvert::Input {
            audio_selectors: Some({
              let mut map = std::collections::HashMap::new();
              map.insert(
                "Audio Selector 1".to_owned(),
                rusoto_mediaconvert::AudioSelector {
                  offset: Some(0),
                  default_selection: Some("DEFAULT".to_owned()),
                  selector_type: Some("TRACK".to_owned()),
                  program_selection: Some(1),
                  ..Default::default()
                },
              );
              map
            }),
            video_selector: Some(rusoto_mediaconvert::VideoSelector {
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
      .await
      .map_err(anyhow::Error::from)?;

    Ok(())
  }
}

impl RekognitionHandler {
  fn message_body(&self, message: String) -> Result<RekognitionNotification, QueueHandlerError> {
    let body: RekognitionNotification =
      serde_json::from_str(&message).map_err(anyhow::Error::from)?;

    Ok(body)
  }
}
