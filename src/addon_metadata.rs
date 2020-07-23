use crate::AddonType;
use crate::Tag;
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
        addon_tags: &[Tag],
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
    pub fn set_tags(&mut self, tag1: Tag, tag2: Tag) {
        self.tags[0] = Self::tag_to_string(&tag1);
        self.tags[1] = Self::tag_to_string(&tag2);
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_type(&self) -> Option<AddonType> {
        Self::string_to_type(&self.addon_type)
    }

    pub fn get_tags(&self) -> (Option<Tag>, Option<Tag>) {
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

    fn string_to_tag(string: &String) -> Option<Tag> {
        match string.to_lowercase().as_str() {
            "fun" => Some(Tag::Fun),
            "roleplay" => Some(Tag::Roleplay),
            "scenic" => Some(Tag::Scenic),
            "movie" => Some(Tag::Movie),
            "realism" => Some(Tag::Realism),
            "cartoon" => Some(Tag::Cartoon),
            "water" => Some(Tag::Water),
            "comic" => Some(Tag::Comic),
            "build" => Some(Tag::Build),
            _ => None,
        }
    }

    fn tag_to_string(tag: &Tag) -> String {
        match tag {
            Tag::Fun => "fun",
            Tag::Roleplay => "roleplay",
            Tag::Scenic => "scenic",
            Tag::Movie => "movie",
            Tag::Realism => "realism",
            Tag::Cartoon => "cartoon",
            Tag::Water => "water",
            Tag::Comic => "comic",
            Tag::Build => "build",
        }
        .to_owned()
    }
}
