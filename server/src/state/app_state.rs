// shared state management

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextMessage {
    pub message_type: MesageType,
    pub content: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    UserPrompt,
    AssistantResponse,
    CliCommand,
    CliOutput,
}

// main struct which manages all app state
pub struct ChatState {
    pub chat_id: UUIDv4,
    pub chat_context: Mutex<Vec<ContextMessage>>,
}

impl ChatState {
    pub fn new(chat_id: UUIDv4) -> Self {
        Self {
            chat_id,
            chat_context: Mutex::new(Vec::new()),
        }
    }

    pub fn add_message_to_state(
        &self,
        message_type: MesageType,
        content: String,
    ) -> Result<(), String> {
        // or should it be .map_err instead of expect? how do you error handle correctly in rust?
        let mut context = prev_context.context.lock().expect("Failed to acquire lock");
        context.push(ContextMessage {
            message_type,
            content,
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }

    pub fn get_full_context(&self) -> Result<String, String> {
        let context = self.chat_context.lock().map_err(|e| e.to_string())?;
        Ok(context
            .iter()
            .map(|msg| {
                format!(
                    "[{}] {}: {}",
                    msg.timestamp,
                    match msg.message_type {
                        MessageType::UserPrompt => "User",
                        MessageType::AssistantResponse => "Assistant",
                        MessageType::CliCommand => "Command",
                        MessageType::CliOutput => "Output",
                    },
                    msg.content
                )
            })
            .collect::<Vec<String>>()
            .join("\n"))
    }
}
