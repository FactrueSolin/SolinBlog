1. 仅需保证代码通过cargo check即可，测试交由用户手动完成
2. /example目录，内部都是参考代码
3. 严格遵守rust的bin和lib最佳实践
4. 已生成 graphify-out/ 知识图谱。回答项目架构、模块关系、代码路径、依赖关系等问题时，优先参考 graphify-out/GRAPH_REPORT.zh.md 与 graphify-out/graph.json；需要精确追踪关系时，使用 graphify query/path/explain，而不是直接全文扫文件。
<info>
项目简述：这是一个AI原生的blog，开放mcp接口供ai连接，直接发布原始html界面。
</info>
