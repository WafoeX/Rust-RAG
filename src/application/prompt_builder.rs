use crate::domain::ports::ChatMessage;
use crate::domain::RetrievedChunk;

pub struct RagPrompt {
    pub system: String,
    pub user: String,
}

pub struct PromptBuilder {
    system_prompt_template: String,
}

impl PromptBuilder {
    pub fn new(system_prompt_template: String) -> Self {
        Self {
            system_prompt_template,
        }
    }

    pub fn build(
        &self,
        question: &str,
        chunks: &[RetrievedChunk],
        history: &[ChatMessage],
    ) -> RagPrompt {
        let mut user_prompt = String::new();

        // Include conversation history for multi-turn continuity
        if !history.is_empty() {
            user_prompt.push_str("之前的对话：\n");
            for msg in history {
                let label = if msg.role == "user" { "用户" } else { "助手" };
                user_prompt.push_str(&format!("{}：{}\n", label, msg.content));
            }
            user_prompt.push('\n');
        }

        // Build context from retrieved chunks
        let mut context = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let score_pct = (chunk.score * 100.0).round();
            context.push_str(&format!(
                "【资料片段 {}】\n文件：{}\n片段序号：{}\n相关度：{}%\n内容：{}\n",
                i + 1,
                chunk.file_name,
                chunk.chunk_index,
                score_pct,
                chunk.content
            ));
        }

        if context.is_empty() {
            user_prompt.push_str(&format!("请回答以下问题：\n\n{}", question));
        } else {
            user_prompt.push_str(&format!(
                "以下是根据问题检索到的相关资料：\n\n{}\n请根据以上资料回答下面的问题：\n\n{}",
                context, question
            ));
        }

        RagPrompt {
            system: self.system_prompt_template.clone(),
            user: user_prompt,
        }
    }
}
