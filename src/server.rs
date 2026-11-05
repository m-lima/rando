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

handle!(bool, f32, f64, u8, u16, u32, u64, i8, i16, i32, i64);
