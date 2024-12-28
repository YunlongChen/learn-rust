use chrono::DateTime;
use core::str;
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::Rng;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method, Response, StatusCode,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, SystemTimeError};
// 加载base64 crate

/// 获取当前时间戳（秒）
pub fn current_timestamp() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

// 对指定的字符串进行URL编码。返回值类型为&Str，URLEncoder.encode(str, "UTF-8").replace("+", "%20").replace("*", "%2A").replace("%7E", "~")
pub fn percent_code(encode_str: &str) -> Cow<'_, str> {
    let encoded = utf8_percent_encode(encode_str, NON_ALPHANUMERIC)
        .to_string()
        .replace("+", "20%")
        .replace("%5F", "_")
        .replace("%2D", "-")
        .replace("%2E", ".")
        .replace("%7E", "~");

    Cow::Owned(encoded) // 返回一个 Cow<str> 可以持有 String 或 &str
}

/// 计算SHA256哈希
pub fn sha256_hex(message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    format!("{:x}", hasher.finalize()).to_lowercase()
}

/// HMAC SHA256
pub fn hmac256(key: &[u8], message: &str) -> Result<Vec<u8>, String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| format!("use data key on sha256 fail:{}", e))?;
    mac.update(message.as_bytes());
    let signature = mac.finalize();
    Ok(signature.into_bytes().to_vec())
}

/// 生成指定长度的随机字符串
pub fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

/// 生成随机字符串作为nonce
pub fn generate_nonce() -> String {
    generate_random_string(32)
}

// 定义 FormData 数据类型
#[derive(Debug, Clone)]
pub enum FormValue {
    String(String),
    // 添加类型：Vec<String>, HashSet<String> 或者 HashMap<String, String> 等
    Vec(Vec<String>),
    HashMap(HashMap<String, String>),
}

// 定义一个body请求体枚举，用于统一处理请求体类型,包含Json,Map，二进制类型
pub enum RequestBody {
    Json(HashMap<String, Value>),         // Json
    Binary(Vec<u8>),                      // Binary
    FormData(HashMap<String, FormValue>), //  FormData
    None,
}

/// 规范化请求
pub async fn call_api(
    client: Client,
    method: Method,
    host: &str,
    canonical_uri: &str,
    query_params: &[(&str, &str)], // 添加$query_params 查询参数
    action: &str,
    version: &str,
    body: RequestBody, // 定义接收请求体body参数 （类型为：Json，map，二进制）
    access_key_id: &str,
    access_key_secret: &str,
) -> Result<String, String> {
    // CanonicalQueryString 构建规范化查询字符串
    let canonical_query_string = build_sored_encoded_query_string(query_params);

    // 处理 body 请求体内容，判断 boby 类型
    let body_content = match &body {
        // 使用引用来避免移动
        RequestBody::Json(body_map) => json!(body_map).to_string(), // 若 body 为map，转化为 &str 类型，存储 body_content 变量中
        RequestBody::Binary(_) => String::new(), // 若 body 为二进制类型这里可以保留空字符串，body_content 变量为空
        RequestBody::FormData(form_data) => {
            let params: Vec<String> = form_data
                .iter()
                .flat_map(|(k, v)| {
                    match v {
                        FormValue::String(s) => {
                            // 当 FormValue 为 String 时
                            vec![format!("{}={}", percent_code(k), percent_code(&s))]
                        }
                        FormValue::Vec(vec) => {
                            // 当 FormValue 为 Vec 时
                            vec.iter()
                                .map(|s| format!("{}={}", percent_code(k), percent_code(s)))
                                .collect::<Vec<_>>()
                        }
                        FormValue::HashMap(map) => {
                            // 当 FormValue 为 HashMap 时
                            map.iter()
                                .map(|(sk, sv)| {
                                    format!("{}={}", percent_code(sk), percent_code(sv))
                                })
                                .collect::<Vec<_>>()
                        }
                    }
                })
                .collect();
            params.join("&") //  组成 key=value&key=value 的形式
        }
        RequestBody::None => String::new(),
    };
    // 打印 body 和 query
    println!("Request Body: {}", body_content);
    println!("Query Params: {:?}", query_params);

    let hashed_request_payload = sha256_hex(&body_content);

    // x-acs-date
    let now_time =
        current_timestamp().map_err(|e| format!("Get current timestamp failed: {}", e))?;
    let datetime = DateTime::from_timestamp(now_time as i64, 0)
        .ok_or_else(|| format!("Get datetime from timestamp failed: {}", now_time))?;
    let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // x-acs-signature-nonce
    let signature_nonce = generate_nonce();

    // 构造请求头
    let mut headers = HeaderMap::new();
    headers.insert("Host", HeaderValue::from_str(host).unwrap());
    headers.insert("x-acs-action", HeaderValue::from_str(action).unwrap());
    headers.insert("x-acs-version", HeaderValue::from_str(version).unwrap());
    headers.insert("x-acs-date", HeaderValue::from_str(&datetime_str).unwrap());
    headers.insert(
        "x-acs-signature-nonce",
        HeaderValue::from_str(&signature_nonce).unwrap(),
    );
    headers.insert(
        "x-acs-content-sha256",
        HeaderValue::from_str(&hashed_request_payload).unwrap(),
    );

    // 签名的消息头
    let sign_header_arr = &[
        "host",
        "x-acs-action",
        "x-acs-content-sha256",
        "x-acs-date",
        "x-acs-signature-nonce",
        "x-acs-version",
    ];

    let sign_headers = sign_header_arr.join(";");
    // 规范化请求头
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n\n{}\n{}",
        method.as_str(),
        canonical_uri,
        canonical_query_string,
        sign_header_arr
            .iter()
            .map(|&header| format!("{}:{}", header, headers[header].to_str().unwrap()))
            .collect::<Vec<_>>()
            .join("\n"),
        sign_headers,
        hashed_request_payload
    );

    // 计算待签名字符串
    let result = sha256_hex(&canonical_request);
    let string_to_sign = format!("ACS3-HMAC-SHA256\n{}", result);
    let signature = hmac256(access_key_secret.as_bytes(), &string_to_sign)?;
    let data_sign = hex::encode(&signature);
    let auth_data = format!(
        "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        access_key_id, sign_headers, data_sign
    );

    headers.insert("Authorization", HeaderValue::from_str(&auth_data).unwrap());

    // 发送请求
    let url = format!("https://{}{}", host, canonical_uri);

    let response = send_request(
        &client,
        method,
        &url,
        headers,
        query_params,  // 接收 query 参数
        &body,         // 接收body请求体参数（包含Json，Map，二进制）
        &body_content, // 接收body请求体参数。当body为map 或者 formDate 的时候，该变量有值；若body参数为二进制，则该变量为空
    )
        .await?;

    // 读取响应
    let (_, res) = read_response(response).await?;
    Ok(res)
}

/// 发送请求
async fn send_request(
    client: &Client,
    method: Method,
    url: &str,
    headers: HeaderMap,
    query_params: &[(&str, &str)], // 接收 query 参数
    body: &RequestBody,            // 接收body请求体参数（包含Json，Map，二进制）
    body_content: &str, // 接收body请求体参数。当body为 map 或者 formDate 的时候，该变量有值；若body参数为二进制，则该变量为空
) -> Result<Response, String> {
    let mut request_builder = client.request(method, url);

    // 添加 query 参数
    if !query_params.is_empty() {
        // 这里使用 query_params 直接作为 query 的参数
        request_builder = request_builder.query(query_params);
    }

    // 添加请求头
    for (k, v) in headers.iter() {
        request_builder = request_builder.header(k, v.clone());
    }

    // 根据 RequestBody 类型设置请求体
    match body {
        // 如果body是二进制，设置 application/octet-stream
        RequestBody::Binary(binary_data) => {
            request_builder = request_builder.header("Content-Type", "application/octet-stream");
            request_builder = request_builder.body(binary_data.clone()); // 移动这里的值
        }
        RequestBody::Json(_) => {
            // 如果body为map，且不为空，转化为Json后存储在 body_content 变量中，设置  application/json; charset=utf-8
            if !body_content.is_empty() {
                request_builder = request_builder.body(body_content.to_string());
                request_builder =
                    request_builder.header("Content-Type", "application/json; charset=utf-8");
            }
        }
        RequestBody::FormData(_) => {
            // 处理 form-data 类型，设置 content-type
            if !body_content.is_empty() {
                request_builder =
                    request_builder.header("Content-Type", "application/x-www-form-urlencoded");
                request_builder = request_builder.body(body_content.to_string());
            }
        }
        RequestBody::None => {
            request_builder = request_builder.body(String::new());
        }
    }

    let request = request_builder
        .build()
        .map_err(|e| format!("build request fail: {}", e))?;

    let response = client
        .execute(request)
        .await
        .map_err(|e| format!("execute request fail: {}", e))?;

    Ok(response)
}

/// 读取响应
pub async fn read_response(result: Response) -> Result<(StatusCode, String), String> {
    let status = result.status();
    let data = result
        .bytes()
        .await
        .map_err(|e| format!("Read response body failed: {}", e))?;
    let res = match str::from_utf8(&data) {
        Ok(s) => s.to_string(),
        Err(_) => return Err("Body contains non UTF-8 characters".to_string()),
    };
    Ok((status, res))
}

/// 构建规范化查询字符串
pub fn build_sored_encoded_query_string(query_params: &[(&str, &str)]) -> String {
    // 按参数名升序排序并使用 BTreeMap 处理重复
    let sorted_query_params: BTreeMap<_, _> = query_params.iter().copied().collect();

    // URI 编码
    let encoded_params: Vec<String> = sorted_query_params
        .into_iter()
        .map(|(k, v)| {
            let encoded_key = percent_code(k);
            let encoded_value = percent_code(v);
            format!("{}={}", encoded_key, encoded_value)
        })
        .collect();

    // 使用 & 连接所有编码后的参数
    encoded_params.join("&")
}
