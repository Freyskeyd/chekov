use uuid::Uuid;
use super::Consistency;

#[derive(Debug, Clone)]
pub struct CommandMetadatas {
    pub command_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub consistency: Consistency,
}

impl std::default::Default for CommandMetadatas {
    fn default() -> Self {
        Self {
            command_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            causation_id: None,
            consistency: Consistency::Eventual,
        }
    }
}
