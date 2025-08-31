//! 数据状态管理
//!
//! 管理所有与业务数据相关的状态，包括域名列表、DNS记录、
//! 提供商信息、过滤器设置等。

use crate::gui::model::domain::DomainStatus;
use crate::gui::model::form::AddDnsField;
use crate::gui::pages::domain::{AddDomainProviderForm, DomainProvider};
use crate::storage::{DnsRecordModal, DomainModal};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;

/// 数据状态结构体
///
/// 包含所有与业务数据相关的状态信息
#[derive(Debug, Clone)]
pub struct DataState {
    /// 域名列表
    pub domain_list: Vec<DomainModal>,

    /// 当前选中的域名
    pub selected_domain: Option<DomainModal>,

    /// DNS提供商列表
    pub domain_providers: Vec<DomainProvider>,

    /// 当前选中的DNS提供商
    pub selected_provider: Option<DomainProvider>,

    /// DNS记录缓存 (域名 -> DNS记录列表)
    pub dns_records_cache: HashMap<usize, Vec<DnsRecordModal>>,

    /// 当前显示的DNS记录
    pub current_dns_records: Vec<DnsRecordModal>,

    /// DNS记录列表（用于兼容原版）
    pub dns_list: Vec<DnsRecordModal>,

    /// 添加DNS记录表单
    pub add_dns_form: AddDnsField,

    /// 过滤器设置
    pub filter: Filter,

    /// 域名统计信息
    pub stats: DomainStats,

    /// 搜索内容
    pub search_content: String,

    /// 是否有数据变更（需要保存）
    pub has_changes: bool,

    /// 最后同步时间
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,

    /// 数据库连接
    pub connection: Option<DatabaseConnection>,

    /// 域名同步状态管理 (域名 -> 是否正在同步)
    pub syncing_domains: HashMap<String, bool>,

    /// 域名统计信息映射 (域名 -> 统计信息)
    pub domain_stats: HashMap<String, DomainStats>,

    /// 添加域名提供商表单
    pub add_domain_provider_form: AddDomainProviderForm,
}

/// 过滤器设置
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    /// 按状态过滤
    pub status: Option<DomainStatus>,

    /// 按提供商过滤
    pub provider: Option<i64>,

    /// 按记录类型过滤
    pub record_type: Option<String>,

    /// 搜索关键词
    pub search_keyword: String,
}

/// 域名统计信息
#[derive(Debug, Clone, Default)]
pub struct DomainStats {
    /// 总域名数
    pub total_domains: usize,

    /// 活跃域名数
    pub active_domains: usize,

    /// 暂停域名数
    pub paused_domains: usize,

    /// 错误域名数
    pub error_domains: usize,

    /// 总DNS记录数
    pub total_dns_records: usize,

    /// DNS记录数量
    pub dns_records_count: usize,

    /// 最后同步时间
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            status: None,
            provider: None,
            record_type: None,
            search_keyword: String::new(),
        }
    }
}

impl Default for DataState {
    fn default() -> Self {
        Self {
            domain_list: Vec::new(),
            selected_domain: None,
            domain_providers: vec![],
            selected_provider: None,
            dns_records_cache: HashMap::new(),
            current_dns_records: Vec::new(),
            dns_list: Vec::new(),
            add_dns_form: AddDnsField::default(),
            filter: Filter::default(),
            stats: DomainStats::default(),
            search_content: String::new(),
            has_changes: false,
            last_sync_time: None,
            connection: None,
            syncing_domains: HashMap::new(),
            domain_stats: HashMap::new(),
            add_domain_provider_form: AddDomainProviderForm::default(),
        }
    }
}

impl DataState {
    /// 创建新的数据状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置域名列表
    pub fn set_domains(&mut self, domains: Vec<DomainModal>) {
        self.domain_list = domains;
        self.update_stats();
        self.has_changes = true;
    }

    /// 添加域名
    pub fn add_domain(&mut self, domain: DomainModal) {
        self.domain_list.push(domain);
        self.update_stats();
        self.has_changes = true;
    }

    /// 删除域名
    pub fn remove_domain(&mut self, domain_id: usize) {
        self.domain_list.retain(|d| d.id as usize != domain_id);
        self.dns_records_cache.remove(&domain_id);

        // 如果删除的是当前选中的域名，清除选择
        if let Some(selected) = &self.selected_domain {
            if selected.id as usize == domain_id {
                self.selected_domain = None;
                self.current_dns_records.clear();
            }
        }

        self.update_stats();
        self.has_changes = true;
    }

    /// 选择域名
    pub fn select_domain(&mut self, domain: DomainModal) {
        // 从缓存中加载DNS记录
        if let Some(records) = self.dns_records_cache.get(&(domain.id as usize)) {
            self.current_dns_records = records.clone();
        } else {
            self.current_dns_records.clear();
        }

        self.selected_domain = Some(domain);
    }

    /// 设置DNS记录
    pub fn set_dns_records(&mut self, domain_id: usize, records: Vec<DnsRecordModal>) {
        self.dns_records_cache
            .insert(domain_id.clone(), records.clone());

        // 如果是当前选中域名的记录，更新当前显示
        if let Some(selected) = &self.selected_domain {
            if selected.id as usize == domain_id {
                self.current_dns_records = records;
            }
        }

        self.update_stats();
        self.has_changes = true;
    }

    /// 添加DNS记录
    pub fn add_dns_record(&mut self, domain_name: usize, record: DnsRecordModal) {
        let records = self
            .dns_records_cache
            .entry(domain_name.clone())
            .or_insert_with(Vec::new);
        records.push(record.clone());

        // 如果是当前选中域名的记录，更新当前显示
        if let Some(selected) = &self.selected_domain {
            if selected.id as usize == domain_name {
                self.current_dns_records.push(record);
            }
        }

        self.update_stats();
        self.has_changes = true;
    }

    /// 删除DNS记录
    pub fn remove_dns_record(&mut self, domain_id: usize, record_id: i64) {
        if let Some(records) = self.dns_records_cache.get_mut(&domain_id) {
            records.retain(|r| r.id != record_id);
        }

        // 如果是当前选中域名的记录，更新当前显示
        if let Some(selected) = &self.selected_domain {
            if selected.id as usize == domain_id {
                self.current_dns_records.retain(|r| r.id != record_id);
            }
        }

        self.update_stats();
        self.has_changes = true;
    }

    /// 设置过滤器
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    /// 获取过滤后的域名列表
    pub fn get_filtered_domains(&self) -> Vec<DomainModal> {
        let mut filtered = self.domain_list.clone();

        // 按搜索关键词过滤
        if !self.filter.search_keyword.is_empty() {
            let keyword = self.filter.search_keyword.to_lowercase();
            filtered.retain(|d| d.name.to_lowercase().contains(&keyword));
        }

        // TODO 按提供商过滤
        if let Some(provider) = &self.filter.provider {
            filtered.retain(|d| true);
        }

        filtered
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        self.stats.total_domains = self.domain_list.len();
        self.stats.total_dns_records = self.dns_records_cache.values().map(|v| v.len()).sum();

        // 这里可以根据实际的域名状态字段来计算活跃、暂停、错误域名数
        // 目前先设置为简单的计算
        self.stats.active_domains = self.domain_list.len();
        self.stats.paused_domains = 0;
        self.stats.error_domains = 0;
    }

    /// 设置搜索内容
    pub fn set_search_content(&mut self, content: String) {
        self.search_content = content;
        self.filter.search_keyword = self.search_content.clone();
    }

    /// 清除所有数据
    pub fn clear(&mut self) {
        self.domain_list.clear();
        self.selected_domain = None;
        self.dns_records_cache.clear();
        self.current_dns_records.clear();
        self.filter = Filter::default();
        self.stats = DomainStats::default();
        self.search_content.clear();
        self.has_changes = false;
        self.last_sync_time = None;
    }

    /// 标记数据已保存
    pub fn mark_saved(&mut self) {
        self.has_changes = false;
    }

    /// 标记数据已更改
    pub fn mark_changed(&mut self) {
        self.has_changes = true;
    }

    /// 设置最后同步时间
    pub fn set_last_sync_time(&mut self, time: chrono::DateTime<chrono::Utc>) {
        self.last_sync_time = Some(time);
    }

    /// 检查是否有未保存的更改
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_changes
    }

    /// 设置域名同步状态
    pub fn set_syncing(&mut self, domain_name: &str, syncing: bool) {
        self.syncing_domains
            .insert(domain_name.to_string(), syncing);
    }

    /// 获取域名同步状态
    pub fn is_syncing(&self, domain_name: &str) -> bool {
        self.syncing_domains
            .get(domain_name)
            .copied()
            .unwrap_or(false)
    }

    /// 清除所有同步状态
    pub fn clear_syncing_states(&mut self) {
        self.syncing_domains.clear();
    }
}
