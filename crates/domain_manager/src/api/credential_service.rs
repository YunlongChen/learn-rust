use crate::api::dns_client::{DnsClient, DnsClientTrait};
use crate::gui::model::domain::DnsProvider;
use crate::gui::types::credential::{
    ApiKeyCredential, Credential, TokenCredential, UsernamePasswordCredential,
};
use anyhow::{anyhow, Result};
use tracing::{error, info};

/// 凭证验证服务
pub struct CredentialService;

impl CredentialService {
    /// 验证凭证是否有效
    pub async fn validate_credentials(
        provider: DnsProvider,
        credential: &Credential,
    ) -> Result<()> {
        match provider {
            DnsProvider::Aliyun => Self::validate_aliyun_credentials(credential).await,
            DnsProvider::CloudFlare => {
                // TODO: 实现 Cloudflare 凭证验证
                info!("Cloudflare 凭证验证暂未实现");
                Ok(())
            }
            DnsProvider::Dnspod => {
                // TODO: 实现 DNSPod 凭证验证
                info!("DNSPod 凭证验证暂未实现");
                Ok(())
            }
            DnsProvider::Tomato
            | DnsProvider::TencentCloud
            | DnsProvider::Aws
            | DnsProvider::Google => {
                // 其他提供商暂不验证
                info!("{} 提供商跳过凭证验证", provider.name());
                Ok(())
            }
        }
    }

    /// 验证阿里云凭证
    async fn validate_aliyun_credentials(credential: &Credential) -> Result<()> {
        let (access_key_id, access_key_secret) = match credential {
            Credential::ApiKey(api_key) => (api_key.api_key.clone(), api_key.api_secret.clone()),
            Credential::Token(token) => {
                // 对于阿里云，token 通常作为 access_key_secret 使用
                ("".to_string(), token.token.clone())
            }
            Credential::UsernamePassword(username_password) => {
                // 阿里云通常使用 API 密钥而不是用户名密码
                (
                    username_password.username.clone(),
                    username_password.password.clone(),
                )
            }
        };

        if access_key_id.is_empty() || access_key_secret.is_empty() {
            return Err(anyhow!("Access Key ID 或 Access Key Secret 不能为空"));
        }

        // 创建 DNS 客户端并验证凭证
        let client = DnsClient::new(
            access_key_id,
            access_key_secret,
            "cn-hangzhou".to_string(), // 默认使用杭州区域
            vec![DnsProvider::Aliyun],
        );

        client.validate_credentials().await.map_err(|e| {
            error!("阿里云凭证验证失败: {:?}", e);
            anyhow!("凭证验证失败: {}", e)
        })
    }

    /// 模拟验证凭证（用于测试和开发环境）
    pub async fn validate_credentials_mock(
        provider: DnsProvider,
        credential: &Credential,
    ) -> Result<()> {
        // 模拟验证过程
        info!("模拟验证 {} 凭证", provider.name());

        // 模拟网络延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 模拟验证逻辑
        match credential {
            Credential::ApiKey(api_key) => {
                if api_key.api_key.contains("test") || api_key.api_secret.contains("test") {
                    Err(anyhow!("测试凭证验证失败"))
                } else if api_key.api_key.is_empty() || api_key.api_secret.is_empty() {
                    Err(anyhow!("API Key 或 API Secret 不能为空"))
                } else {
                    Ok(())
                }
            }
            Credential::Token(token) => {
                if token.token.contains("test") {
                    Err(anyhow!("测试token验证失败"))
                } else if token.token.is_empty() {
                    Err(anyhow!("Token 不能为空"))
                } else {
                    Ok(())
                }
            }
            Credential::UsernamePassword(username_password) => {
                if username_password.username.contains("test")
                    || username_password.password.contains("test")
                {
                    Err(anyhow!("测试用户名密码验证失败"))
                } else if username_password.username.is_empty()
                    || username_password.password.is_empty()
                {
                    Err(anyhow!("用户名或密码不能为空"))
                } else {
                    Ok(())
                }
            }
        }
    }
}
