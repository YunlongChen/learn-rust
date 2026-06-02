use crate::gui::pages::domain::{AddDomainProviderForm, DomainProvider};

/// 域名服务商管理页面的状态
#[derive(Debug, Clone)]
pub struct ProviderPageState {
    /// 服务商列表数据
    pub providers: Vec<DomainProvider>,

    /// 添加/编辑服务商表单
    pub form: AddDomainProviderForm,

    /// 表单是否可见
    pub form_visible: bool,

    /// 当前正在编辑的服务商ID (None表示新增模式)
    pub editing_provider_id: Option<i64>,

    /// 当前正在确认删除的服务商ID
    pub deleting_provider_id: Option<i64>,

    /// 是否正在加载数据
    pub is_loading: bool,

    /// 鼠标悬停的服务商ID
    pub hovered_provider_id: Option<i64>,
}

impl Default for ProviderPageState {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            form: AddDomainProviderForm::default(),
            form_visible: false,
            editing_provider_id: None,
            deleting_provider_id: None,
            is_loading: false,
            hovered_provider_id: None,
        }
    }
}

impl ProviderPageState {
    pub fn new() -> Self {
        Self::default()
    }
}
