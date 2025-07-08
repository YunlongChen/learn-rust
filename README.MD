### 使用sea-orm-cli 生成实体类

```shell
 sea-orm-cli generate entity -s domain --with-serde both --model-extra-attributes 'serde(rename_all = "camelCase")' --date-time-crate chrono -o ./src/entity
```