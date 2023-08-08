
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "eth_feeHistory", "params": ["2", "latest",[]]}' https://evm.astar.network
{"jsonrpc":"2.0","result":{"oldestBlock":"0x34021c","baseFeePerGas":["0x3b9aca00","0x3b9aca00","0x3b9aca00"],"gasUsedRatio":[0.059556733333333334,0.0743376],"reward":null},"id":1}
