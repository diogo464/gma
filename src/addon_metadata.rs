use crate::AddonType;
use crate::AddonTag;
use nanoserde::{self, DeJson, SerJson};

#[derive(Debug, SerJson, DeJson)]
pub struct AddonMetadata {
    title: Option<String>,
    description: String,
    #[nserde(rename = "type")]
    addon_type: String,
    tags: Vec<String>,
}

#[allow(dead_code)]
impl AddonMetadata {
    pub fn new(
        title: String,
        description: String,
        addon_type: &AddonType,
        addon_tags: &[AddonTag],
    ) -> Self {
        let mut string_tags = Vec::new();
        for t in addon_tags {
            string_tags.push(Self::tag_to_string(&t))
        }
        Self {
            title: Some(title),
            description,
            addon_type: Self::type_to_string(&addon_type),
            tags: string_tags,
        }
    }

    pub fn from_json(json: &str) -> Option<Self> {
        Self::deserialize_json(json).ok()
    }

    pub fn to_json(&self) -> String {
        self.serialize_json()
    }

    pub fn set_description(&mut self, desc: String) {
        self.description = desc;
    }

    pub fn set_type(&mut self, addon_type: AddonType) {
        self.addon_type = Self::type_to_string(&addon_type)
    }
    pub fn set_tags(&mut self, tag1: AddonTag, tag2: AddonTag) {
        self.tags[0] = Self::tag_to_string(&tag1);
        self.tags[1] = Self::tag_to_string(&tag2);
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_type(&self) -> Option<AddonType> {
        Self::string_to_type(&self.addon_type)
    }

    pub fn get_tags(&self) -> (Option<AddonTag>, Option<AddonTag>) {
        let opt_t1 = self.tags.get(0).map(|s| Self::string_to_tag(s));
        let opt_t2 = self.tags.get(1).map(|s| Self::string_to_tag(s));
        (opt_t1.unwrap_or(None), opt_t2.unwrap_or(None))
    }

    fn string_to_type(string: &String) -> Option<AddonType> {
        let lowcase = string.to_lowercase();
        match lowcase.as_str() {
            "gamemode" => Some(AddonType::Gamemode),
            "map" => Some(AddonType::Map),
            "weapon" => Some(AddonType::Weapon),
            "vehicle" => Some(AddonType::Vehicle),
            "npc" => Some(AddonType::NPC),
            "entity" => Some(AddonType::Entity),
            "tool" => Some(AddonType::Tool),
            "effects" => Some(AddonType::Effects),
            "model" => Some(AddonType::Model),
            "servercontent" => Some(AddonType::ServerContent),
            _ => None,
        }
    }

    fn type_to_string(ty: &AddonType) -> String {
        match ty {
            AddonType::Gamemode => "gamemode",
            AddonType::Map => "map",
            AddonType::Weapon => "weapon",
            AddonType::Vehicle => "vehicle",
            AddonType::NPC => "npc",
            AddonType::Entity => "entity",
            AddonType::Tool => "tool",
            AddonType::Effects => "effects",
            AddonType::Model => "model",
            AddonType::ServerContent => "servercontent",
        }
        .to_owned()
    }

    fn string_to_tag(string: &String) -> Option<AddonTag> {
        match string.to_lowercase().as_str() {
            "fun" => Some(AddonTag::Fun),
            "roleplay" => Some(AddonTag::Roleplay),
            "scenic" => Some(AddonTag::Scenic),
            "movie" => Some(AddonTag::Movie),
            "realism" => Some(AddonTag::Realism),
            "cartoon" => Some(AddonTag::Cartoon),
            "water" => Some(AddonTag::Water),
            "comic" => Some(AddonTag::Comic),
            "build" => Some(AddonTag::Build),
            _ => None,
        }
    }

    fn tag_to_string(tag: &AddonTag) -> String {
        match tag {
            AddonTag::Fun => "fun",
            AddonTag::Roleplay => "roleplay",
            AddonTag::Scenic => "scenic",
            AddonTag::Movie => "movie",
            AddonTag::Realism => "realism",
            AddonTag::Cartoon => "cartoon",
            AddonTag::Water => "water",
            AddonTag::Comic => "comic",
            AddonTag::Build => "build",
        }
        .to_owned()
    }
}
