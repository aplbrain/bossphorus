# Feature Parity with BossDB

<!-- emoji for your copypasting convenience: ✅🔴🔜 -->

| Feature \\ Data Manager | `ChunkedFileDataManager` | `BossDBRelayDataManager` |
|-------------------------|-------------------------------|--------------------------|
| `get_data` | ✅ | ✅ |
| `put_data` | ✅ | 🔴¹ |
| `channel_metadata` | ✅ | 🔴 |

> ¹ `BossDBRelayDataManager.put_data` is not currently on the roadmap because it would involve writing data to a BossDB source as an anonymous (`public`) user.



| Key | Emoji |
|-----|-------|
| ✅ | Supported |
| 🔴 | Not supported |
| 🔜 | Roadmap |
