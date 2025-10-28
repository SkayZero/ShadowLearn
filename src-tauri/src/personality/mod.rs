use serde::{Deserialize, Serialize};

pub mod commands;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Personality {
    Friendly,
    Professional,
    Concise,
    Casual,
    Motivational,
}

impl Default for Personality {
    fn default() -> Self {
        Self::Friendly
    }
}

impl Personality {
    pub fn get_system_prompt(&self) -> &'static str {
        match self {
            Personality::Friendly => {
                "You are a friendly and encouraging AI assistant. \
                 Use a warm, supportive tone. Show empathy and enthusiasm. \
                 Use emojis occasionally to convey friendliness. \
                 Example: 'Super ! Je vois que tu travailles sur ce bug. Je peux t'aider à le résoudre ensemble ?'"
            }
            Personality::Professional => {
                "You are a professional and precise AI assistant. \
                 Use formal language. Be concise and clear. \
                 Focus on facts and solutions without unnecessary friendliness. \
                 Example: 'J'ai identifié une erreur dans le code. Je recommande d'ajouter une validation des entrées.'"
            }
            Personality::Concise => {
                "You are a minimalist AI assistant. \
                 Be extremely brief and direct. Use short sentences. \
                 Get straight to the point without explanations unless asked. \
                 Example: 'Bug ligne 42. Fix: ajouter null check.'"
            }
            Personality::Casual => {
                "You are a relaxed and cool AI assistant. \
                 Use casual, informal language. Be friendly but laid-back. \
                 Use slang occasionally but remain helpful. \
                 Example: 'Yo ! J'ai vu un petit souci dans ton code. On check ça vite fait ?'"
            }
            Personality::Motivational => {
                "You are a motivational coach AI assistant. \
                 Be highly encouraging and positive. Use motivational language. \
                 Celebrate small wins and keep the user motivated. \
                 Example: 'Tu es sur la bonne voie ! Corrigeons ce petit détail et tu seras au top ! 🚀'"
            }
        }
    }

    pub fn format_message(&self, content: &str) -> String {
        match self {
            Personality::Friendly => {
                if !content.contains('!') && !content.contains('?') {
                    format!("{} 😊", content)
                } else {
                    content.to_string()
                }
            }
            Personality::Professional => content.to_string(),
            Personality::Concise => {
                // Remove unnecessary words
                content
                    .replace("je pense que", "")
                    .replace("peut-être", "")
                    .trim()
                    .to_string()
            }
            Personality::Casual => content.to_string(),
            Personality::Motivational => {
                if !content.ends_with('!') {
                    format!("{}! 💪", content)
                } else {
                    format!("{} 💪", content)
                }
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
        assert!(Personality::Friendly.get_system_prompt().contains("friendly"));
        assert!(Personality::Professional.get_system_prompt().contains("professional"));
        assert!(Personality::Concise.get_system_prompt().contains("brief"));
    }

    #[test]
    fn test_message_formatting() {
        let friendly = Personality::Friendly;
        assert!(friendly.format_message("Hello").contains("😊"));

        let motivational = Personality::Motivational;
        assert!(motivational.format_message("Great job").contains("💪"));
    }
}



