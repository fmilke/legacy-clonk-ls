use tracing::info;
use tower_lsp::lsp_types::ClientCapabilities;

#[derive(Debug, Default)]
pub struct MarkdownInfo;

impl MarkdownInfo {

    pub fn from_cabpilities(caps: &ClientCapabilities) -> Self {
        match caps.general {
            Some(ref gen_caps) => {
                match gen_caps.markdown {
                    Some(ref markdown_caps) => {
                        if let Some(ref allowed) = markdown_caps.allowed_tags {
                            info!("Supported tags: {:?}", allowed);
                        } else {
                            info!("No tags supported");
                        }

                        Default::default()
                    },
                    _ => Default::default(),
                }
            },
            _ => Default::default(),
        }
    }
}
