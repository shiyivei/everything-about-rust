// 如何开始一个项目？先确定需要几个模块，再确定模块和模块之间的接口，最后实现细节
// 按照如上设计思路:
// 1.kv-server: 适应高性能的场景，优先考虑TCP协议，对其他支持要灵活
// 客户端和服务器的结构或者协议、服务器和命令处理流程的接口、服务器和存储的接口
// 客户端的命令通过 protobuf 定义
// 请求中包含了数据，它是一些命令，要去做解析。我们把所有类型的请求放在一个枚举数据结构中
// 需要把数据结构和数据库的操作方式对应起来
// 再需要把查询结果返回去
// 整个过程应该支持异步编程

mod error;
mod network;
mod pb;
mod service;
mod storage;
pub use error::KvError;
pub use network::*;
pub use pb::abi::*;
pub use service::*;
pub use storage::*;
