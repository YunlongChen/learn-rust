//! 背景组件模块
//! 
//! 提供应用程序背景图片显示功能，支持不同类型的背景和透明度调节

use crate::configs::gui_config::BackgroundType;
use iced::widget::{container, svg, Container, Svg};
use iced::{Element, Length};
use std::path::Path;

/// 背景组件结构体
#[derive(Debug, Clone)]
pub struct Background {
    /// 背景类型
    background_type: BackgroundType,
    /// 透明度 (0.0 - 1.0)
    opacity: f32,
}

impl Background {
    /// 创建新的背景组件
    /// 
    /// # 参数
    /// * `background_type` - 背景类型
    /// * `opacity` - 透明度 (0.0 - 1.0)
    /// 
    /// # 返回
    /// 背景组件实例
    pub fn new(background_type: BackgroundType, opacity: f32) -> Self {
        Self {
            background_type,
            opacity: opacity.clamp(0.0, 1.0),
        }
    }

    /// 获取背景图片路径
    /// 
    /// # 返回
    /// 背景图片的文件路径，如果没有背景则返回None
    fn get_background_path(&self) -> Option<String> {
        match self.background_type {
            BackgroundType::None => None,
            BackgroundType::ChinaRed => Some("resources/backgrounds/china_red_background.svg".to_string()),
            BackgroundType::QipaoGirl => Some("resources/backgrounds/qipao_girl_background.svg".to_string()),
        }
    }

    /// 渲染背景组件
    /// 
    /// # 类型参数
    /// * `Message` - 消息类型
    /// * `Theme` - 主题类型
    /// 
    /// # 返回
    /// 背景元素
    pub fn view<Message, Theme>(&self) -> Element<'static, Message, Theme>
    where
        Message: 'static + Clone,
        Theme: 'static + iced::widget::svg::Catalog + iced::widget::container::Catalog,
    {
        if let Some(path) = self.get_background_path() {
            // 检查文件是否存在
            if Path::new(&path).exists() {
                let svg_handle = svg::Handle::from_path(&path);
                let background_svg = Svg::new(svg_handle)
                    .width(Length::Fill)
                    .height(Length::Fill);

                Container::new(background_svg)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            } else {
                // 如果文件不存在，返回透明容器
                Container::new(iced::widget::Space::new(Length::Fill, Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        } else {
            // 没有背景时返回透明容器
            Container::new(iced::widget::Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    /// 设置背景类型
    /// 
    /// # 参数
    /// * `background_type` - 新的背景类型
    pub fn set_background_type(&mut self, background_type: BackgroundType) {
        self.background_type = background_type;
    }

    /// 设置透明度
    /// 
    /// # 参数
    /// * `opacity` - 新的透明度值 (0.0 - 1.0)
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// 获取当前背景类型
    /// 
    /// # 返回
    /// 当前的背景类型
    pub fn background_type(&self) -> BackgroundType {
        self.background_type.clone()
    }

    /// 获取当前透明度
    /// 
    /// # 返回
    /// 当前的透明度值
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new(BackgroundType::None, 1.0)
    }
}