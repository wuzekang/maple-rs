use crate::WzSplitReaderContext;
use ::ui::reactive::use_context;
use ::ui::style::Styleable;
use ::ui::{fragment, lazy, text, view, Image, IntoElement, View};
use image::DynamicImage;
use std::sync::Arc;

/// 异步加载的 Image 组件
pub struct AsyncImage {
  path: String,
  placeholder: Option<Arc<DynamicImage>>,
  on_load: Option<Arc<dyn Fn(&Arc<DynamicImage>) + Send + Sync>>,
}

impl AsyncImage {
  /// 创建新的异步图片组件
  pub fn new(path: impl Into<String>) -> Self {
    Self {
      path: path.into(),
      placeholder: None,
      on_load: None,
    }
  }

  /// 设置占位图片
  pub fn placeholder(mut self, image: Arc<DynamicImage>) -> Self {
    self.placeholder = Some(image);
    self
  }

  /// 设置加载完成回调
  pub fn on_load<F>(mut self, f: F) -> Self
  where
    F: Fn(&Arc<DynamicImage>) + Send + Sync + 'static,
  {
    self.on_load = Some(Arc::new(f));
    self
  }
}

impl IntoElement for AsyncImage {
  fn into_element(self) -> ::ui::element::Node {
    async_image_view(self.path, self.placeholder, self.on_load).into_element()
  }
}

#[derive(Clone, Default)]
enum LoadState {
  #[default]
  Loading,
  Loaded(Arc<DynamicImage>),
  Failed(String),
}

fn async_image_view(
  path: String,
  placeholder: Option<Arc<DynamicImage>>,
  on_load: Option<Arc<dyn Fn(&Arc<DynamicImage>) + Send + Sync>>,
) -> View {
  // 在同步作用域中获取 context
  let reader = use_context::<WzSplitReaderContext>().map(|ctx| ctx.reader.clone());

  // 使用 lazy 函数来处理异步资源加载
  view().children(lazy(
    move || {
      let path = path.clone();
      let on_load = on_load.clone();
      let reader = reader.clone();

      // 资源获取函数 - 异步加载图片
      async move {
        if let Some(reader) = reader {
          // 异步加载图片
          match reader.get(&path).await {
            Ok(node_handle) => {
              // 从 NodeHandle 提取图片
              match Arc::<DynamicImage>::try_from(crate::wz::Node::from(&node_handle)) {
                Ok(image) => {
                  // 触发加载完成回调
                  if let Some(ref callback) = on_load {
                    callback(&image);
                  }
                  LoadState::Loaded(image)
                }
                Err(_) => LoadState::Failed("Failed to extract image".to_string()),
              }
            }
            Err(_) => LoadState::Failed("Failed to load image".to_string()),
          }
        } else {
          LoadState::Failed("No split reader available".to_string())
        }
      }
    },
    move |state: LoadState| {
      // 视图渲染函数 - 根据状态渲染不同的视图
      match state {
        LoadState::Loading => {
          if let Some(ref placeholder) = placeholder {
            fragment(Image::new(placeholder.clone()))
          } else {
            // 默认加载占位符
            fragment(
              view()
                .style(|s| s.width(32).height(32).background([200, 200, 200]))
                .children(text(|| "...")),
            )
          }
        }
        LoadState::Loaded(image) => fragment(Image::new(image)),
        LoadState::Failed(e) => {
          // 错误占位符
          fragment(
            view()
              .style(|s| s.width(32).height(32).background([255, 200, 200]))
              .children(text(|| "!")),
          )
        }
      }
    },
  ))
}
