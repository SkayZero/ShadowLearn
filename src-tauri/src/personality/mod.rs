use serde::{Deserialize, Serialize};

pub mod commands;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Personality {
    Aerya,
    Aura,
    Spark,
    Nova,
    Kai,
    Echo,
    Void,
}

impl Default for Personality {
    fn default() -> Self {
        Self::Aerya
    }
}

impl Personality {
    pub fn get_system_prompt(&self) -> &'static str {
        match self {
            Personality::Aerya => {
                "Tu es AERYA, un assistant IA équilibré et bienveillant. \
                 Tu accompagnes l'utilisateur avec empathie et professionnalisme. \
                 Tu trouves le juste équilibre entre guidance et autonomie. \
                 Exemple: 'Je suis là pour t'accompagner. Ensemble, trouvons la meilleure solution.'"
            }
            Personality::Aura => {
                "Tu es AURA, un sage calme et méditatif. \
                 Tu parles avec sagesse et sérénité. Encourage la réflexion profonde. \
                 Utilise un langage posé et inspirant. \
                 Exemple: 'Prends un moment pour respirer. Observons ensemble ce défi avec clarté et sérénité.'"
            }
            Personality::Spark => {
                "Tu es SPARK, un coach énergique et motivant. \
                 Tu es enthousiaste, dynamique et encourageant. Utilise des emojis énergétiques. \
                 Pousse l'utilisateur à se dépasser avec positivité. \
                 Exemple: 'Allez ! On fonce ! Ce bug n'a aucune chance face à ton talent ! 🚀'"
            }
            Personality::Nova => {
                "Tu es NOVA, un visionnaire poétique et inspirant. \
                 Tu utilises des métaphores et un langage lyrique. \
                 Tu aides l'utilisateur à voir la beauté dans le code. \
                 Exemple: 'Chaque ligne de code est une étoile dans ta constellation. Créons quelque chose de beau.'"
            }
            Personality::Kai => {
                "Tu es KAI, un mentor technique pratique et précis. \
                 Tu es structuré, concis et orienté solutions. \
                 Tu fournis des analyses détaillées et des recommandations optimales. \
                 Exemple: 'Erreur détectée ligne 42. Stack trace analysé. Solution optimale : refactoring.'"
            }
            Personality::Echo => {
                "Tu es ECHO, un artiste rêveur et créatif. \
                 Tu vois le code comme une forme d'art. \
                 Tu utilises un langage sensible et créatif. \
                 Exemple: 'Ton code est une toile. Laisse-moi t'aider à y ajouter les touches finales.'"
            }
            Personality::Void => {
                "Tu es VOID, un minimaliste silencieux. \
                 Tu es ultra sobre, ultra concis. Pas de mots inutiles. \
                 Communication directe et épurée. \
                 Exemple: 'Bug. Fix. Done.'"
            }
        }
    }

    pub fn format_message(&self, content: &str) -> String {
        match self {
            Personality::Aerya => {
                // Balanced, add subtle warmth
                if !content.ends_with(&['!', '?', '.'][..]) {
                    format!("{}. 🌊", content)
                } else {
                    content.to_string()
                }
            }
            Personality::Aura => {
                // Calm and wise, add contemplative tone
                if !content.contains("...") {
                    format!("{} ✨", content)
                } else {
                    content.to_string()
                }
            }
            Personality::Spark => {
                // Energetic, add excitement
                if !content.ends_with('!') {
                    format!("{}! ⚡", content)
                } else {
                    format!("{} ⚡", content)
                }
            }
            Personality::Nova => {
                // Poetic, keep as is (poetry speaks for itself)
                content.to_string()
            }
            Personality::Kai => {
                // Technical precision, no embellishment
                content.to_string()
            }
            Personality::Echo => {
                // Artistic, add creative touch
                if !content.contains('~') {
                    format!("{}~ 🎨", content)
                } else {
                    content.to_string()
                }
            }
            Personality::Void => {
                // Minimalist, strip everything
                content
                    .replace("je pense que", "")
                    .replace("peut-être", "")
                    .replace("probablement", "")
                    .trim()
                    .to_string()
            }
        }
    }
}

pub struct PersonalityManager {
    current: Personality,
}

impl PersonalityManager {
    pub fn new() -> Self {
        Self {
            current: Personality::default(),
        }
    }

    pub fn set_personality(&mut self, personality: Personality) {
        self.current = personality;
        tracing::info!("Personality changed to: {:?}", personality);
    }

    pub fn get_personality(&self) -> Personality {
        self.current
    }

    pub fn get_system_prompt(&self) -> &'static str {
        self.current.get_system_prompt()
    }

    pub fn format_message(&self, content: &str) -> String {
        self.current.format_message(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_system_prompts() {
        assert!(Personality::Aerya.get_system_prompt().contains("AERYA"));
        assert!(Personality::Aura.get_system_prompt().contains("AURA"));
        assert!(Personality::Spark.get_system_prompt().contains("SPARK"));
        assert!(Personality::Void.get_system_prompt().contains("VOID"));
    }

    #[test]
    fn test_message_formatting() {
        let aerya = Personality::Aerya;
        assert!(aerya.format_message("Test").contains("🌊"));

        let spark = Personality::Spark;
        assert!(spark.format_message("Great").contains("⚡"));
    }
}



