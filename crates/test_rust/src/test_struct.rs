use std::any::Any;

///
/// 结构体
///
/// Rust 很多地方受 JavaScript 影响，在实例化结构体的时候用 JSON 对象的 key: value 语法来实现定义：
///
/// Rust 中的结构体（Struct）与元组（Tuple）都可以将若干个类型不一定相同的数据捆绑在一起形成整体，
/// 但结构体的每个成员和其本身都有一个名字，这样访问它成员的时候就不用记住下标了。
/// 元组常用于非定义的多值传递，而结构体用于规范常用的数据结构。结构体的每个成员叫做"字段"。
pub fn test_struct() {
    let nation = String::from("China");
    let mut domain_info = DomainInfo {
        domain: String::from("www.chenyunlong.cn"),
        found: 1996,
        // 如果正在实例化的结构体有字段名称和现存变量名称一样的，可以简化书写
        nation,
        unit: UnitStruct {},
    };

    print_domain_info(&domain_info);

    domain_info.nation.push_str("-changed");

    print_domain_info(&domain_info);

    // 你想要新建一个结构体的实例，其中大部分属性需要被设置成与现存的一个结构体属性一样，仅需更改其中的一两个字段的值，可以使用结构体更新语法：
    let domain_info2 = DomainInfo {
        domain: String::from("www.chenyunlong.cn"),
        // 如果正在实例化的结构体有字段名称和现存变量名称一样的，可以简化书写,这个地方使用的是上一个结构体当前实例的值
        ..domain_info
    };

    print_domain_info(&domain_info2);

    let create_domain = DomainInfo::create(
        String::from("test.cn"),
        226,
        String::from("123123"),
        UnitStruct {},
    );

    print_domain_info(&create_domain);

    let unit_struct = UnitStruct {};

    println!("{:#?}", unit_struct);
}

/// 输出Domain信息
fn print_domain_info(domain: &DomainInfo) {
    println!(
        "struct.domain：{},\tstruct.found：{}\t来自nation：{}",
        domain.domain, domain.found, domain.nation
    );
    println!("struct.domain：{:#?}", domain);
    println!("struct.summary：{}", domain.summary());
    println!("struct.type_id：{:?}", domain.unit.type_id());
}

#[derive(Debug)]
struct DomainInfo {
    domain: String,
    found: i32,
    nation: String,
    unit: UnitStruct,
}

impl DomainInfo {
    fn summary(&self) -> String {
        let mut value = self.domain.clone();
        value.push_str("\t");
        value.push_str(&*self.nation.clone());
        value
    }
}

/// 结构体 impl 块可以写几次，效果相当于它们内容的拼接！
impl DomainInfo {
    fn create(domain: String, found: i32, nation: String, unit: UnitStruct) -> DomainInfo {
        DomainInfo {
            domain,
            found,
            nation,
            unit,
        }
    }
}

#[derive(Debug)]
struct UnitStruct;
