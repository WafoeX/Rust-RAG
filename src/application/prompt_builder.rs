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

    pub fn build(&self, question: &str, chunks: &[RetrievedChunk]) -> RagPrompt {
        let mut context = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            context.push_str(&format!(
                "【资料片段 {}】\n文件：{}\n片段序号：{}\n内容：{}\n",
                i + 1,
                chunk.file_name,
                chunk.chunk_index,
                chunk.content
            ));
        }

        let user_prompt = if context.is_empty() {
            format!("请回答以下问题：\n\n{}", question)
        } else {
            format!(
                "以下是根据问题检索到的相关资料：\n\n{}\n请根据以上资料回答下面的问题：\n\n{}",
                context, question
            )
        };

        RagPrompt {
            system: self.system_prompt_template.clone(),
            user: user_prompt,
        }
    }
}
