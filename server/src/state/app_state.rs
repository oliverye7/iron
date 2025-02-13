// shared state management
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextMessage {
    pub message_type: MessageType,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserChatPreferences {
    pub depth: u16,
    pub autorun_readonly: bool,
    pub autorun_all: bool,
}

impl Default for UserChatPreferences {
    fn default() -> Self {
        Self {
            depth: 5,               // Default depth
            autorun_readonly: true, // Default readonly autorun
            autorun_all: false,     // Default rwx autorun
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MessageType {
    UserPrompt,
    AssistantResponse,
    ReadOnlyCliCommand,
    WriteExecuteCliCommand,
    CliOutput,
    UserCancelCmd,
    UserAckCmd,
}

// let CliCommandType be a strict subset of MessageType
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
pub enum CliCommandType {
    ReadOnlyCliCommand,
    WriteExecuteCliCommand,
}

impl From<CliCommandType> for MessageType {
    fn from(value: CliCommandType) -> Self {
        match value {
            CliCommandType::ReadOnlyCliCommand => MessageType::ReadOnlyCliCommand,
            CliCommandType::WriteExecuteCliCommand => MessageType::WriteExecuteCliCommand,
        }
    }
}

// main struct which manages all app state
#[derive(Debug)]
pub struct ChatState {
    pub chat_id: Uuid,
    pub chat_context: Mutex<Vec<ContextMessage>>,
    pub user_preferences: Mutex<UserChatPreferences>,
}

pub type SharedChatState = Arc<ChatState>;

impl ChatState {
    pub fn new(chat_id: Uuid) -> Self {
        Self {
            chat_id,
            chat_context: Mutex::new(Vec::new()),
            user_preferences: Mutex::new(UserChatPreferences::default()),
        }
    }

    pub fn add_message_to_state(
        &self,
        message_type: MessageType,
        content: String,
    ) -> Result<(), String> {
        // or should it be .map_err instead of expect? how do you error handle correctly in rust?
        let mut context = self.chat_context.lock().expect("Failed to acquire lock");
        context.push(ContextMessage {
            message_type,
            content,
            timestamp: Some(chrono::Utc::now()),
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
                    msg.timestamp
                        .map_or("No timestamp".to_string(), |ts| ts.to_string()),
                    match msg.message_type {
                        MessageType::UserPrompt => "User",
                        MessageType::AssistantResponse => "Assistant",
                        MessageType::ReadOnlyCliCommand => "ReadOnlyCliCommand",
                        MessageType::WriteExecuteCliCommand => "WriteExecuteCliCommand",
                        MessageType::CliOutput => "Output",
                        MessageType::UserCancelCmd => "Cancel",
                        MessageType::UserAckCmd => "Ack",
                    },
                    msg.content
                )
            })
            .collect::<Vec<String>>()
            .join("\n"))
    }
}
