/// This enum defines the possible web pages to be opened.
#[derive(Debug, Clone)]
pub enum WebPage {
    /// GitHub repository.
    Repo,
    ///  website main page.
    // Website,
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
            WebPage::Repo => "https://github.com/GyulyVGC/sniffnet",
            // WebPage::Website => "https://www.sniffnet.net",
            WebPage::WebsiteSponsor => "https://www.sniffnet.net/sponsor",
            WebPage::WebsiteDownload => "https://www.sniffnet.net/download",
            WebPage::WebsiteNews => "https://www.sniffnet.net/news",
            WebPage::Issues => "https://github.com/GyulyVGC/sniffnet/issues",
            WebPage::IssueLanguages => "https://github.com/GyulyVGC/sniffnet/issues/60",
            WebPage::Wiki => "https://github.com/GyulyVGC/sniffnet/wiki",
            WebPage::MyGitHub => "https://github.com/GyulyVGC",
        }
    }
}

#[test]
fn test_web_page() {
    assert_eq!(
        WebPage::Repo.get_url(),
        "https://github.com/GyulyVGC/sniffnet"
    );
    assert_eq!(
        WebPage::WebsiteDownload.get_url(),
        "https://www.sniffnet.net/download"
    );
}
