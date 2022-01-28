use super::Consistency;
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadatas_can_be_initialize() {
        let _meta = CommandMetadatas {
            command_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            consistency: Consistency::Eventual,
        };
    }
}
