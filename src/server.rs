use crate::logger;

macro_rules! handle {
    ($type: ty) => {
        paste::item! {
            async fn [<rand_$type>]() -> String {
                format!("{}", rand::random::<$type>())
            }
        }
    };
    ($type: ty, $($rest: ty),+) => {
        handle!($type);
        handle!($($rest),+);
    };
}

macro_rules! handle_range {
    ($type: ty) => {
        paste::item! {
            async fn [<rand_$type>](range: axum::extract::Query<Range<$type>>) -> String {
                let value = if let Some(bounds) = range.bounds() {
                    rand::random_range(bounds)
                } else {
                    rand::random()
                };
                format!("{value}")
            }
        }

        impl Range<$type> {
            fn bounds(&self) -> Option<std::ops::RangeInclusive<$type>> {
                if let Some(from) = self.0.from {
                    let to = self.0.to.unwrap_or(<$type>::MAX);
                    let to = if self.0.inclusive { to } else { to - 1 };
                    Some(from..=to)
                } else if let Some(to) = self.0.to {
                    let from = <$type>::MIN;
                    let to = if self.0.inclusive { to } else { to - 1 };
                    Some(from..=to)
                } else {
                    None
                }
            }
        }

        impl<'de> serde::de::Deserialize<'de> for Range<$type> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let inner = InnerRange::deserialize(deserializer)?;

                let from = inner.from.unwrap_or(<$type>::MIN);
                let to = inner.to.unwrap_or(<$type>::MAX);

                if inner.inclusive {
                    if from > to {
                        return Err(<D::Error as serde::de::Error>::custom(
                            "Value of `from` cannot be greater than `to`",
                        ));
                    }
                } else {
                    if from >= to {
                        return Err(<D::Error as serde::de::Error>::custom(
                            "Value of `to` must be less than `from`",
                        ));
                    }
                }

                Ok(Self(inner))
            }
        }
    };
    ($type: ty, $($rest: ty),+) => {
        handle_range!($type);
        handle_range!($($rest),+);
    };
}

macro_rules! route {
    ($router: ident, $type: ty) => {
        paste::item! {
            $router.route(concat!("/", stringify!($type)), axum::routing::get([<rand_$type>]))
        }
    };
    ($router: ident, $type: ty, $($rest: ty),+) => {{
        let router = route!($router, $type);
        let router = route!(router, $($rest),+);
        router
    }};
}

pub fn make_routes() -> axum::Router {
    let router = axum::Router::new();
    route!(router, bool, f32, f64, u8, u16, u32, u64, i8, i16, i32, i64).layer(logger::Logger)
}

handle!(bool, f32, f64);
handle_range!(u8, u16, u32, u64, i8, i16, i32, i64);

#[derive(Debug, serde::Deserialize)]
struct InnerRange<T> {
    from: Option<T>,
    to: Option<T>,
    #[serde(default)]
    inclusive: bool,
}

#[repr(transparent)]
#[derive(Debug)]
struct Range<T>(InnerRange<T>);
