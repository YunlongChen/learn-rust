/// This enum defines the possible web pages to be opened.
#[derive(Debug, Clone)]
pub enum WebPage {
    /// GitHub repository.
    Repo,
    ///  website main page.
    Website,
    ///  website/download page.
    WebsiteDownload,
    ///  website/news page.
    WebsiteNews,
    ///  website/sponsor page.
    WebsiteSponsor,
    /// issues
    Issues,
    /// issue #60 on GitHub
    IssueLanguages,
    /// Wiki
    Wiki,
    /// My GitHub profile
    MyGitHub,
}

impl WebPage {
    pub fn get_url(&self) -> &str {
        match self {
            WebPage::Repo => "https://github.com/YunlongChen/learn-rust",
            WebPage::Website => "https://www.stanic.xyz",
            WebPage::WebsiteSponsor => "https://www.stanic.xyz/sponsor",
            WebPage::WebsiteDownload => "https://github.com/YunlongChen/learn-rust/releases",
            WebPage::WebsiteNews => "https://www.stanic.xyz/news",
            WebPage::Issues => "https://github.com/YunlongChen/learn-rust/issues",
            WebPage::IssueLanguages => "https://github.com/YunlongChen/learn-rust/issues/60",
            WebPage::Wiki => "https://github.com/YunlongChen/learn-rust/wiki",
            WebPage::MyGitHub => "https://github.com/YunlongChen",
        }
    }
}

#[test]
fn test_web_page() {
    assert_eq!(
        WebPage::Repo.get_url(),
        "https://github.com/YunlongChen/learn-rust"
    );
    assert_eq!(
        WebPage::WebsiteDownload.get_url(),
        "https://github.com/YunlongChen/learn-rust/releases"
    );
}
