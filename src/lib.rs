use std::sync::Mutex;

use cookie_store::Cookie;
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;
use reqwest::Url;

pub struct ReqwestCookieStore {
    jar: Mutex<cookie_store::CookieStore>,
}

impl CookieStore for ReqwestCookieStore {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item=&HeaderValue>, url: &Url) {
        // handle set-cookie headers
        let mut mutex_guard = self.jar.lock().unwrap();
        for header_value in cookie_headers {
            let s = header_value.to_str().unwrap();
            mutex_guard.parse(s, url).expect("TODO: panic message");
        }
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let mutex_guard = self.jar.lock().unwrap();
        let cookies = mutex_guard.get_request_values(url)
            .map(|(key, value)| {
                format!("{}={}", key, value)
            })
            .collect::<Vec<_>>();
        if cookies.is_empty() {
            return None;
        }
        let str = cookies.join("; ");
        let header_value = HeaderValue::from_str(str.as_str()).unwrap();
        Some(header_value)
    }
}

impl Default for ReqwestCookieStore {
    fn default() -> Self {
        ReqwestCookieStore {
            jar: Mutex::new(Default::default())
        }
    }
}

impl ReqwestCookieStore {
    pub fn get_jar(&self) -> &Mutex<cookie_store::CookieStore> {
        &self.jar
    }

    pub fn get_cookie(&self, name: &str) -> Vec<Cookie<'static>> {
        let jar = self.get_jar().lock().unwrap();
        jar.iter_unexpired()
            .filter(|cookie| cookie.name() == name)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn remove_cookie(&self, name: &str) -> Vec<Cookie<'static>> {
        let mut jar = self.get_jar().lock().unwrap();
        let to_remove = jar.iter_unexpired()
            .filter(|cookie| cookie.name() == name)
            .cloned()
            .collect::<Vec<_>>();
        let mut result = vec![];
        for cookie in to_remove {
            if let Some(removed) = jar.remove(cookie.domain().unwrap(), cookie.path().unwrap(), cookie.name()) {
                result.push(removed);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use reqwest::Client;

    use crate::ReqwestCookieStore;

    #[test]
    fn it_works() {
        let cookie_jar = ReqwestCookieStore::default();
        let cookie_arc = Arc::new(cookie_jar);
        let client = Client::builder()
            .cookie_provider(cookie_arc.clone());
    }
}
