use domain_client::{call_api, RequestBody};
use reqwest::{Client, Method};
use serde_json::json;
use std::collections::HashMap;
use std::env;

/**
 * 签名示例，您需要根据实际情况替换main方法中的示例参数。
 * ROA接口和RPC接口只有canonicalUri取值逻辑是完全不同，其余内容都是相似的。
 * 通过API元数据获取请求方法（methods）、请求参数名称（name）、请求参数类型（type）、请求参数位置（in），并将参数封装到SignatureRequest中。
 * 1. 请求参数在元数据中显示"in":"query"，通过queryParam传参。
 * 2. 请求参数在元数据中显示"in": "body"，通过body传参。
 * 2. 请求参数在元数据中显示"in": "formData"，通过body传参。
*/
#[tokio::main]
async fn main() {
    // 创建 HTTP 客户端
    let client = Client::new();
    // env::var()表示通过环境变量获取Access Key ID和Access Key Secret

    let access_key_id = env::var("ALIBABA_CLOUD_ACCESS_KEY_ID").expect("Cannot get access key id.");
    let access_key_secret =
        env::var("ALIBABA_CLOUD_ACCESS_KEY_SECRET").expect("Cannot get access key id.");

    // RPC接口请求示例一：请求参数"in":"query"   POST
    let host = "domain.aliyuncs.com"; // endpoint
    let canonical_uri = "/"; // RPC接口无资源路径，故使用正斜杠（/）作为CanonicalURI
    let action = "QueryDomainList"; // API名称
    let version = "2018-01-29"; // API版本号
                                // RegionId在元数据中显示的类型是String，"in":"query"，必填
    let query_params = &[
        ("RegionId", "cn-hangzhou"),
        ("PageNum", "1"),
        ("PageSize", "10"),
    ];
    // 构建查询参数  InstanceId的在元数据中显示的类型是array，"in":"query"，非必填
    // let region_id = "cn-hangzhou";
    // let instance_ids = vec![
    //     "i-bp11ht4h2kd1XXXXXXXX",
    //     "i-bp16maz3h3xgXXXXXXXX",
    //     "i-bp10r67hmsllXXXXXXXX",
    // ];
    // // // 将 instance_ids 转换为逗号分隔的字符串
    // let instance_id_str = instance_ids.join(",");
    // // 创建查询参数时，开始的时候添加 RegionId
    // let mut query: Vec<(&str, &str)> = vec![("RegionId", region_id), ("InstanceId", &instance_id_str)];
    // let query_params = &query[..];

    // RPC接口请求示例二：请求参数"in":"body"  POST
    // let host = "ocr-api.cn-hangzhou.aliyuncs.com";
    // let canonical_uri = "/";
    // let action = "RecognizeGeneral";
    // let version = "2021-07-07";
    // // // 上传文件
    // let binary_data = std::fs::read("C:\\Users\\issuser\\Desktop\\img\\001.png").expect("读文件失败"); // 参数必须要直接传文件二进制

    // RPC接口请求示例三：请求参数"in": "formData"  POST
    // let host = "mt.aliyuncs.com";
    // let canonical_uri = "/";
    // let action = "TranslateGeneral";
    // let version = "2018-10-12";
    // // Context在元数据中显示的类型是String，"in":"query"，非必填
    // let query_params = &[("Context", "早上")];
    // // FormatType、SourceLanguage、TargetLanguage等参数，在元数据中显示"in":"formData"
    // let mut body = HashMap::new();  //  HashMap<String, FormValue>   FormValue  可支持Vec<String>, HashSet<String> 或者 HashMap<String, String> ...，更多类型可在FormValue枚举中添加
    // body.insert(String::from("FormatType"),FormValue::String(String::from("text")));
    // body.insert(String::from("SourceLanguage"),FormValue::String(String::from("zh")));
    // body.insert(String::from("TargetLanguage"),FormValue::String(String::from("en")));
    // body.insert(String::from("SourceText"),FormValue::String(String::from("你好")));
    // body.insert(String::from("Scene"),FormValue::String(String::from("general")));

    // ROA接口POST请求  API:CreateCluster创建集群
    // 定义API请求常量
    // let host = "cs.cn-beijing.aliyuncs.com";
    // let canonical_uri = "/clusters";
    // let action = "CreateCluster";
    // let version = "2015-12-15";

    // // 设置请求体参数
    let mut body = HashMap::new(); //  HashMap<String, Value>  Value支持类型：Value::String("test".to_string()) // String  Value::Number(serde_json::Number::from(42)) // Number  Value::Bool(true) // Boolean  Value::Null // Null  Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]) //Array json!({"nested_key": "nested_value"})
    body.insert(String::from("PageNum"), json!("1"));
    body.insert(String::from("PageSize"), json!("10"));

    // ROA接口GET请求   API:DeleteCluster  查询指定集群的关联资源
    // let host = "cs.cn-beijing.aliyuncs.com"; // endpoint
    // // 拼接资源路径
    // let uri = format!("/clusters/{}/resources", percent_code("c1311ba68f3af45f39ee3f4d2XXXXXXXX").as_ref());
    // let canonical_uri = uri.as_str(); // 资源路径  转化为&Str类型
    // let action = "DescribeClusterResources";   // API名称
    // let version = "2015-12-15"; // API版本号
    // // 设置查询参数
    // let query_params = &[("with_addon_resources", if true { "true" } else { "false" })];  // "true" or "false"

    // ROA接口DELETE请求   API:DeleteCluster  DELETE请求删除一个按量付费的集群
    // let host = "cs.cn-beijing.aliyuncs.com";
    // let uri = format!("/clusters/{}", percent_code("c72b778e79d3647cdb95c8b86XXXXXXXX").as_ref());
    // let canonical_uri = uri.as_str(); // 资源路径转化为&Str类型
    // let action = "DeleteCluster";
    // let version = "2015-12-15";

    // 调用接口
    match call_api(
        client.clone(),
        Method::GET,   // 请求方法：GET，DELETE，PUT，POST
        host,          // endpoint
        canonical_uri, // 资源路径
        // &[],                                                        // 当查询参数为空时，query_params 设置为  &[]
        query_params, // 当查询参数不为空时， query_params 设置为 &[("K", "V")]
        action,
        version,
        // RequestBody::None, // 当body参数类型为空时，使用 RequestBody:: None 设置为 None；
        RequestBody::Json(body), // 当body参数类型为Map时，使用 RequestBody::Json 传递 Map
        // RequestBody::Binary(binary_data),                           // 当body参数类型为二进制时，使用 RequestBody::Binary 传递二进制数据
        // RequestBody::FormData(body),                                // 当body参数类型为 formDate 时，使用 RequestBody::FormData 传递 FormData
        access_key_id.as_str(),
        access_key_secret.as_str(),
    )
    .await
    {
        Ok(response) => {
            println!("响应信息: {}", response)
        }
        Err(error) => eprintln!("异常: {}", error),
    }
}
