use crate::models::SkillLevel;
use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::ser::{Serialize, SerializeStruct};
use serde::Serialize as DeriveSerialize;

pub trait Game: Send + Sync + 'static {
  fn skill_levels(&self) -> Vec<SkillLevel>;
  fn name(&self) -> String;
  fn id(&self) -> String;
}

impl Serialize for dyn Game {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_struct("Game", 3)?;
    state.serialize_field("name", &self.name())?;
    state.serialize_field("id", &self.id())?;
    state.serialize_field("skill_levels", &self.skill_levels())?;
    state.end()
  }
}

#[derive(DeriveSerialize, JsonSchema)]
pub struct GameSchema {
  name: String,
  id: String,
  skill_levels: Vec<SkillLevel>,
}

impl JsonSchema for dyn Game {
  fn schema_name() -> String {
    "Game".to_string()
  }

  fn json_schema(gen: &mut SchemaGenerator) -> Schema {
    let mut schema = SchemaObject {
      instance_type: Some(InstanceType::Object.into()),
      ..Default::default()
    };

    let obj = schema.object();

    obj.required.insert("skill_levels".to_owned());
    obj.required.insert("name".to_owned());
    obj.required.insert("id".to_owned());

    obj.properties.insert(
      "skill_levels".to_owned(),
      <Vec<SkillLevel>>::json_schema(gen),
    );

    obj
      .properties
      .insert("name".to_owned(), <String>::json_schema(gen));

    obj
      .properties
      .insert("id".to_owned(), <String>::json_schema(gen));

    schema.into()
  }

  fn is_referenceable() -> bool {
    true
  }
}
