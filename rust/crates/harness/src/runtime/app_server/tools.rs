//! Launch-local HTTP tool declaration. The credential is only serialized into
//! the private transport; neither argv nor a native config file carries it.
use crate::NativeMcpDeclaration;
use serde::Serialize;
use zeroize::Zeroizing;

pub(super) struct Prepared<'a> {
    url: &'a str,
    authorization: Zeroizing<String>,
}

impl<'a> Prepared<'a> {
    pub(super) fn new(declaration: &'a NativeMcpDeclaration) -> Self {
        let (url, key) = declaration.http_credentials();
        Self {
            url,
            authorization: Zeroizing::new(format!("Bearer {}", key.expose())),
        }
    }

    pub(super) fn config(&self) -> Config<'_> {
        Config {
            mcp_servers: Servers {
                chapchoo: Server {
                    url: self.url,
                    http_headers: Headers {
                        authorization: self.authorization.as_str(),
                    },
                    required: true,
                    default_tools_approval_mode: "prompt",
                },
            },
            code_mode: CodeMode {
                direct_only_tool_namespaces: ["mcp__chapchoo"],
            },
            features: Features {
                code_mode_only_strict_3p_tools: false,
            },
        }
    }
}

#[derive(Serialize)]
pub(super) struct Config<'a> {
    mcp_servers: Servers<'a>,
    code_mode: CodeMode,
    features: Features,
}

#[derive(Serialize)]
struct Servers<'a> {
    chapchoo: Server<'a>,
}

#[derive(Serialize)]
struct Server<'a> {
    url: &'a str,
    http_headers: Headers<'a>,
    required: bool,
    default_tools_approval_mode: &'static str,
}

#[derive(Serialize)]
struct Headers<'a> {
    #[serde(rename = "Authorization")]
    authorization: &'a str,
}

#[derive(Serialize)]
struct CodeMode {
    direct_only_tool_namespaces: [&'static str; 1],
}

#[derive(Serialize)]
struct Features {
    code_mode_only_strict_3p_tools: bool,
}
