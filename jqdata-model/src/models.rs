use serde_derive::*;
use serde::Deserialize;
use jqdata_derive::*;
use bigdecimal::BigDecimal;
use std::io::Read;
use crate::{Result, Error};

/// Request
/// 
/// Generic request for all JQData APIs,
/// uses serde flatten attribute to construct
/// plain json with different fields.
/// common fields are "token" and "method".
/// specific ones are all passed by payload field.
#[derive(Debug, Serialize, Deserialize)]
pub struct Request<P> {
    token: String,
    method: String,
    #[serde(flatten)]
    payload: P,
}

/// helper trait to expose API method for each request,
/// used by jqdata-derive crate
pub trait HasMethod {
    fn method(&self) -> String;
}

impl<P: HasMethod> Request<P> {
    pub fn new(token: String, payload: P) -> Self {
        Request{
            token,
            method: payload.method(),
            payload,
        }
    }
}

/// helper trait to consume response body and construct
/// the result
pub trait BodyConsumer<T> 
where for<'de> T: Deserialize<'de>
{
    fn consume_body<R: Read>(body: R) -> Result<T>;
}

/// consume body as csv
/// used by jqdata-derive crate
pub trait CsvListBodyConsumer {
    type Output: for<'de> Deserialize<'de>;

    fn consume<R: Read>(body: R) -> Result<Vec<Self::Output>> {
        let mut reader = csv::ReaderBuilder::new()
        // .has_headers(true)
        .from_reader(body);
        // consume the first row as header
        let header_cols: Vec<&str> = reader.headers()?.into_iter().collect();
        if header_cols.is_empty() {
            return Err(Error::Server("empty response body returned".to_owned()));
        }
        let first_col = header_cols.first().cloned().unwrap();
        if first_col.starts_with("error") {
            return Err(Error::Server(first_col.to_owned()));
        }
        let mut rs = Vec::new();
        for r in reader.deserialize() {
            let s: Self::Output = r?;
            rs.push(s);
        }
        Ok(rs)
    }
}

/// consume body as lines
/// used by jqdata-derive crate
pub trait LineBodyConsumer {
    fn consume<R: Read>(body: R) -> Result<Vec<String>> {
        use std::io::BufRead;
        let reader = std::io::BufReader::new(body);
        let mut rs = Vec::new();
        for line in reader.lines() {
            rs.push(line?);
        }
        Ok(rs)
    }
}

/// consume body as single result
/// used by jqdata-derive crate
pub trait SingleBodyConsumer<T> where T: std::str::FromStr, Error: From<T::Err> {
    fn consume<R: Read>(body: R) -> Result<T> {
        let mut body = body;
        let mut vec = Vec::new();
        std::io::copy(&mut body, &mut vec)?;
        let s = String::from_utf8(vec)?;
        let result = s.parse()?;
        Ok(result)
    }
}

/// consume body as json
/// used by jqdata-derive crate
pub trait JsonBodyConsumer {
    type Output: for<'de> Deserialize<'de>;

    fn consume<R: Read>(body: R) -> Result<Self::Output> {
        let result = serde_json::from_reader(body)?;
        Ok(result)
    }
}

/// 证券类型
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityKind {
    Stock,
    Fund,
    Index,
    Futures,
    #[serde(rename = "etf")]
    ETF,
    #[serde(rename = "lof")]
    LOF,
    #[serde(rename = "fja")]
    FJA,
    #[serde(rename = "fjb")]
    FJB,
    #[serde(rename = "QDII_fund")]
    QDIIFund,
    OpenFund,
    BondFund,
    StockFund,
    MoneyMarketFund,
    MixtureFund,
    Options,
}

/// 证券信息
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Security {
    pub code: String,
    pub display_name: String,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(rename = "type")]
    pub kind: SecurityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// 获取平台支持的所有股票、基金、指数、期货信息
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_all_securities")]
#[consume(format = "csv", type = "Security")]
pub struct GetAllSecurities {
    pub code: SecurityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// 获取股票/基金/指数的信息
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_security_info")]
#[consume(format = "csv", type = "Security")]
pub struct GetSecurityInfo {
    pub code: String,
}

/// 获取一个指数给定日期在平台可交易的成分股列表
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_index_stocks")]
#[consume(format = "line")]
pub struct GetIndexStocks {
    pub code: String,
    pub date: String,
}

/// 获取指定日期上交所、深交所披露的的可融资标的列表
/// 查询日期，默认为前一交易日
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_margincash_stocks")]
#[consume(format = "line")]
pub struct GetMargincashStocks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// 获取指定日期区间内的限售解禁数据
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_locked_shares")]
#[consume(format = "csv", type = "LockedShare")]
pub struct GetLockedShares {
    pub code: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockedShare {
    pub day: String,
    pub code: String,
    pub num: BigDecimal,
    pub rate1: BigDecimal,
    pub rate2: BigDecimal,
}

/// 获取指数成份股给定日期的权重数据，每月更新一次
/// code: 代表指数的标准形式代码， 形式：指数代码.交易所代码，例如"000001.XSHG"。
/// date: 查询权重信息的日期，形式："%Y-%m-%d"，例如"2018-05-03"；
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_index_weights")]
#[consume(format = "csv", type = "IndexWeight")]
pub struct GetIndexWeights {
    pub code: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexWeight {
    pub code: String,
    pub display_name: String,
    pub date: String,
    pub weight: BigDecimal,
}

/// 按照行业分类获取行业列表
/// code：行业代码
/// sw_l1: 申万一级行业
/// sw_l2: 申万二级行业
/// sw_l3: 申万三级行业
/// jq_l1: 聚宽一级行业
/// jq_l2: 聚宽二级行业
/// zjw: 证监会行业
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_industries")]
#[consume(format = "csv", type = "IndustryIndex")]
pub struct GetIndustries {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndustryIndex {
    pub index: String,
    pub name: String,
    pub start_date: String,
}

/// 查询股票所属行业
/// 参数：
/// code：证券代码
/// date：查询的日期
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_industry")]
#[consume(format = "csv", type = "Industry")]
pub struct GetIndustry {
    pub code: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Industry {
    pub industry: String,
    pub industry_code: String,
    pub industry_name: String,
}

/// 获取在给定日期一个行业的所有股票
/// 参数：
/// code: 行业编码
/// date: 查询日期
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_industry_stocks")]
#[consume(format = "line")]
pub struct GetIndustryStocks {
    pub code: String,
    pub date: String,
}

/// 获取在给定日期一个概念板块的所有股票
/// 参数：
/// code: 概念板块编码
/// date: 查询日期,
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_concepts")]
#[consume(format = "csv", type = "Concept")]
pub struct GetConcepts {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Concept {
    pub code: String,
    pub name: String,
    pub start_date: String,
}

/// 获取在给定日期一个概念板块的所有股票
/// 参数：
/// code: 概念板块编码
/// date: 查询日期,
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_concept_stocks")]
#[consume(format = "line")]
pub struct GetConceptStocks {
    pub code: String,
    pub date: String,
}

/// 获取指定日期范围内的所有交易日
/// 参数：
/// date: 开始日期
/// end_date: 结束日期
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_trade_days")]
#[consume(format = "line")]
pub struct GetTradeDays {
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

/// 获取所有交易日
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_all_trade_days")]
#[consume(format = "line")]
pub struct GetAllTradeDays {}

/// 获取一只股票在一个时间段内的融资融券信息
/// 参数：
/// code: 股票代码
/// date: 开始日期
/// end_date: 结束日期
/// 返回：
/// date: 日期
/// sec_code: 股票代码
/// fin_value: 融资余额(元）
/// fin_buy_value: 融资买入额（元）
/// fin_refund_value: 融资偿还额（元）
/// sec_value: 融券余量（股）
/// sec_sell_value: 融券卖出量（股）
/// sec_refund_value: 融券偿还量（股）
/// fin_sec_value: 融资融券余额（元）
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_mtss")]
#[consume(format = "csv", type = "Mtss")]
pub struct GetMtss {
    pub code: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mtss {
    pub date: String,
    pub sec_code: String,
    pub fin_value: BigDecimal,
    pub fin_refund_value: BigDecimal,
    pub sec_value: BigDecimal,
    pub sec_sell_value: BigDecimal,
    pub sec_refund_value: BigDecimal,
    pub fin_sec_value: BigDecimal,
}

/// 获取一只股票在一个时间段内的资金流向数据，仅包含股票数据，不可用于获取期货数据
/// 参数：
/// code: 股票代码
/// date: 开始日期
/// end_date: 结束日期
/// 返回：
/// date: 日期
/// sec_code: 股票代码
/// change_pct: 涨跌幅(%)
/// net_amount_main: 主力净额(万): 主力净额 = 超大单净额 + 大单净额
/// net_pct_main: 主力净占比(%): 主力净占比 = 主力净额 / 成交额
/// net_amount_xl: 超大单净额(万): 超大单：大于等于50万股或者100万元的成交单
/// net_pct_xl: 超大单净占比(%): 超大单净占比 = 超大单净额 / 成交额
/// net_amount_l: 大单净额(万): 大单：大于等于10万股或者20万元且小于50万股或者100万元的成交单
/// net_pct_l: 大单净占比(%): 大单净占比 = 大单净额 / 成交额
/// net_amount_m: 中单净额(万): 中单：大于等于2万股或者4万元且小于10万股或者20万元的成交单
/// net_pct_m: 中单净占比(%): 中单净占比 = 中单净额 / 成交额
/// net_amount_s: 小单净额(万): 小单：小于2万股或者4万元的成交单
/// net_pct_s: 小单净占比(%): 小单净占比 = 小单净额 / 成交额
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_money_flow")]
#[consume(format = "csv", type = "MoneyFlow")]
pub struct GetMoneyFlow {
    pub code: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoneyFlow {
    pub date: String,
    pub sec_code: String,
    pub change_pct: BigDecimal,
    pub net_amount_main: BigDecimal,
    pub net_pct_main: BigDecimal,
    pub net_amount_xl: BigDecimal,
    pub net_pct_xl: BigDecimal,
    pub net_amount_l: BigDecimal,
    pub net_pct_l: BigDecimal,
    pub net_amount_m: BigDecimal,
    pub net_pct_m: BigDecimal,
    pub net_amount_s: BigDecimal,
    pub net_pct_s: BigDecimal,
}

/// 获取指定日期区间内的龙虎榜数据
/// 参数：
/// code: 股票代码
/// date: 开始日期
/// end_date: 结束日期
/// 返回：
/// code: 股票代码
/// day: 日期
/// direction: ALL 表示『汇总』，SELL 表示『卖』，BUY 表示『买』
/// abnormal_code: 异常波动类型
/// abnormal_name: 异常波动名称
/// sales_depart_name: 营业部名称
/// rank: 0 表示汇总， 1~5 表示买一到买五， 6~10 表示卖一到卖五
/// buy_value: 买入金额
/// buy_rate: 买入金额占比(买入金额/市场总成交额)
/// sell_value: 卖出金额
/// sell_rate: 卖出金额占比(卖出金额/市场总成交额)
/// net_value: 净额(买入金额 - 卖出金额)
/// amount: 市场总成交额
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_billboard_list")]
#[consume(format = "csv", type = "BillboardStock")]
pub struct GetBillboardList {
    pub code: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillboardStock {
    pub code: String,
    pub day: String,
    pub direction: String,
    pub rank: i32,
    pub abnormal_code: String,
    pub abnormal_name: String,
    pub sales_depart_name: String,
    pub buy_value: BigDecimal,
    pub buy_rate: BigDecimal,
    pub sell_value: BigDecimal,
    pub sell_rate: BigDecimal,
    pub total_value: BigDecimal,
    pub net_value: BigDecimal,
    pub amount: BigDecimal,
}

/// 获取某期货品种在指定日期下的可交易合约标的列表
/// 参数：
/// code: 期货合约品种，如 AG (白银)
/// date: 指定日期
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_future_contracts")]
#[consume(format = "line")]
pub struct GetFutureContracts {
    pub code: String,
    pub date: String,
}

/// 获取主力合约对应的标的
/// 参数：
/// code: 期货合约品种，如 AG (白银)
/// date: 指定日期参数，获取历史上该日期的主力期货合约
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_dominant_future")]
#[consume(format = "line")]
pub struct GetDominantFuture {
    pub code: String,
    pub date: String,
}

/// 获取单个基金的基本信息
/// 参数：
/// code: 基金代码
/// date: 查询日期， 默认日期是今天。
/// 返回：
/// fund_name: 基金全称
/// fund_type: 基金类型
/// fund_establishment_day: 基金成立日
/// fund_manager: 基金管理人及基本信息
/// fund_management_fee: 基金管理费
/// fund_custodian_fee: 基金托管费
/// fund_status: 基金申购赎回状态
/// fund_size: 基金规模（季度）
/// fund_share: 基金份额（季度）
/// fund_asset_allocation_proportion: 基金资产配置比例（季度）
/// heavy_hold_stocks: 基金重仓股（季度）
/// heavy_hold_stocks_proportion: 基金重仓股占基金资产净值比例（季度）
/// heavy_hold_bond: 基金重仓债券（季度）
/// heavy_hold_bond_proportion: 基金重仓债券占基金资产净值比例（季度）
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_fund_info")]
#[consume(format = "json", type = "FundInfo")]
pub struct GetFundInfo {
    pub code: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundInfo {
    pub fund_name: String,
    pub fund_type: String,
    pub fund_establishment_day: String,
    pub fund_manager: String,
    pub fund_management_fee: String,
    pub fund_custodian_fee: String,
    pub fund_status: String,
    pub fund_size: String,
    pub fund_share: BigDecimal,
    pub fund_asset_allocation_proportion: String,
    pub heavy_hold_stocks: Vec<String>,
    pub heavy_hold_stocks_proportion: BigDecimal,
    pub heavy_hold_bond: Vec<String>,
    pub heavy_hold_bond_proportion: BigDecimal,
}

/// 获取最新的 tick 数据
/// 参数：
/// code: 标的代码， 支持股票、指数、基金、期货等。 不可以使用主力合约和指数合约代码。
/// 返回：
/// time: 时间
/// current: 当前价
/// high: 截至到当前时刻的日内最高价
/// low: 截至到当前时刻的日内最低价
/// volume: 累计成交量
/// money: 累计成交额
/// position: 持仓量，期货使用
/// a1_v~a5_v: 五档卖量
/// a1_p~a5_p: 五档卖价
/// b1_v~b5_v: 五档买量
/// b1_p~b5_p: 五档买价
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_current_tick")]
#[consume(format = "csv", type = "Tick")]
pub struct GetCurrentTick {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
    pub time: BigDecimal,
    pub current: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub volumn: BigDecimal,
    pub money: BigDecimal,
    pub position: BigDecimal,
    pub a1_v: BigDecimal,
    pub a2_v: BigDecimal,
    pub a3_v: BigDecimal,
    pub a4_v: BigDecimal,
    pub a5_v: BigDecimal,
    pub a1_p: BigDecimal,
    pub a2_p: BigDecimal,
    pub a3_p: BigDecimal,
    pub a4_p: BigDecimal,
    pub a5_p: BigDecimal,
    pub b1_v: BigDecimal,
    pub b2_v: BigDecimal,
    pub b3_v: BigDecimal,
    pub b4_v: BigDecimal,
    pub b5_v: BigDecimal,
    pub b1_p: BigDecimal,
    pub b2_p: BigDecimal,
    pub b3_p: BigDecimal,
    pub b4_p: BigDecimal,
    pub b5_p: BigDecimal,
}

/// 获取多标的最新的 tick 数据
/// 参数：
/// code: 标的代码， 多个标的使用,分隔。每次请求的标的必须是相同类型。标的类型包括： 股票、指数、场内基金、期货、期权
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_current_ticks")]
#[consume(format = "csv", type = "Tick")]
pub struct GetCurrentTicks {
    pub code: String,
}

/// 获取基金净值/期货结算价等
/// 参数：
/// code: 证券代码
/// date: 开始日期
/// end_date: 结束日期
/// 返回：
/// date: 日期
/// is_st: 是否是ST，是则返回 1，否则返回 0。股票使用
/// acc_net_value: 基金累计净值。基金使用
/// unit_net_value: 基金单位净值。基金使用
/// futures_sett_price: 期货结算价。期货使用
/// futures_positions: 期货持仓量。期货使用
/// adj_net_value: 场外基金的复权净值。场外基金使用
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_extras")]
#[consume(format = "csv", type = "Extra")]
pub struct GetExtras {
    pub code: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    pub date: String,
    pub is_st: Option<i8>,
    pub acc_net_value: Option<f64>,
    pub unit_net_value: Option<f64>,
    pub futures_sett_price: Option<f64>,
    pub futures_positions: Option<f64>,
    pub adj_net_value: Option<f64>,
}

/// 获取各种时间周期的bar数据，bar的分割方式与主流股票软件相同， 同时还支持返回当前时刻所在 bar 的数据。get_price 与 get_bars 合并为一个函数
/// 参数：
/// code: 证券代码
/// count: 大于0的整数，表示获取bar的条数，不能超过5000
/// unit: bar的时间单位, 支持如下周期：1m, 5m, 15m, 30m, 60m, 120m, 1d, 1w, 1M。其中m表示分钟，d表示天，w表示周，M表示月
/// end_date：查询的截止时间，默认是今天
/// fq_ref_date：复权基准日期，该参数为空时返回不复权数据
/// 返回：
/// date: 日期
/// open: 开盘价
/// close: 收盘价
/// high: 最高价
/// low: 最低价
/// volume: 成交量
/// money: 成交额
/// 当unit为1d时，包含以下返回值:
/// paused: 是否停牌，0 正常；1 停牌
/// high_limit: 涨停价
/// low_limit: 跌停价
/// avg: 当天均价
/// pre_close：前收价
/// 当code为期货和期权时，包含以下返回值:
/// open_interest 持仓量
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_price")]
#[consume(format = "csv", type = "Price")]
pub struct GetPrice {
    pub date: String,
    pub count: u32,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fq_ref_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    pub date: String,
    pub open: BigDecimal,
    pub close: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub volume: BigDecimal,
    pub money: BigDecimal,
    pub paused: Option<u8>,
    pub high_limit: Option<f64>,
    pub low_limit: Option<f64>,
    pub avg: Option<f64>,
    pub pre_close: Option<f64>,
    pub open_interest: Option<f64>,
}

/// 指定开始时间date和结束时间end_date时间段，获取行情数据
/// 参数：
/// code: 证券代码
/// unit: bar的时间单位, 支持如下周期：1m, 5m, 15m, 30m, 60m, 120m, 1d, 1w, 1M。其中m表示分钟，d表示天，w表示周，M表示月
/// date: 开始时间，不能为空，格式2018-07-03或2018-07-03 10:40:00，如果是2018-07-03则默认为2018-07-03 00:00:00
/// end_date：结束时间，不能为空，格式2018-07-03或2018-07-03 10:40:00，如果是2018-07-03则默认为2018-07-03 23:59:00
/// fq_ref_date：复权基准日期，该参数为空时返回不复权数据
/// 注：当unit是1w或1M时，第一条数据是开始时间date所在的周或月的行情。当unit为分钟时，第一条数据是开始时间date所在的一个unit切片的行情。
/// 最大获取1000个交易日数据
/// 返回：
/// date: 日期
/// open: 开盘价
/// close: 收盘价
/// high: 最高价
/// low: 最低价
/// volume: 成交量
/// money: 成交额
/// 当unit为1d时，包含以下返回值:
/// paused: 是否停牌，0 正常；1 停牌
/// high_limit: 涨停价
/// low_limit: 跌停价
/// 当code为期货和期权时，包含以下返回值:
/// open_interest 持仓量
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_price_period")]
#[consume(format = "csv", type = "Price")]
pub struct GetPricePeriod {
    pub code: String,
    pub unit: String,
    pub date: String,
    pub end_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fq_ref_date: Option<String>,
}

/// 获取tick数据
/// 股票部分， 支持 2010-01-01 至今的tick数据，提供买五卖五数据
/// 期货部分， 支持 2010-01-01 至今的tick数据，提供买一卖一数据。 如果要获取主力合约的tick数据，可以先使用get_dominant_future获取主力合约对应的标的
/// 期权部分，支持 2017-01-01 至今的tick数据，提供买五卖五数据
/// 参数：
/// code: 证券代码
/// count: 取出指定时间区间内前多少条的tick数据，如不填count，则返回end_date一天内的全部tick
/// end_date: 结束日期，格式2018-07-03或2018-07-03 10:40:00
/// skip: 默认为true，过滤掉无成交变化的tick数据；
/// 当skip=false时，返回的tick数据会保留从2019年6月25日以来无成交有盘口变化的tick数据。
/// 由于期权成交频率低，所以建议请求期权数据时skip设为false
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_ticks")]
#[consume(format = "csv", type = "Tick")]
pub struct GetTicks {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    pub end_date: String,
    pub skip: bool,
}

/// 按时间段获取tick数据
/// 股票部分， 支持 2010-01-01 至今的tick数据，提供买五卖五数据
/// 期货部分， 支持 2010-01-01 至今的tick数据，提供买一卖一数据。 如果要获取主力合约的tick数据，可以先使用get_dominant_future获取主力合约对应的标的
/// 期权部分，支持 2017-01-01 至今的tick数据，提供买五卖五数据
/// 参数：
/// code: 证券代码
/// date: 开始时间，格式2018-07-03或2018-07-03 10:40:00
/// end_date: 结束时间，格式2018-07-03或2018-07-03 10:40:00
/// skip: 默认为true，过滤掉无成交变化的tick数据；
/// 当skip=false时，返回的tick数据会保留从2019年6月25日以来无成交有盘口变化的tick数据。
/// 注：
/// 如果时间跨度太大、数据量太多则可能导致请求超时，所有请控制好data-end_date之间的间隔！
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_ticks_period")]
#[consume(format = "csv", type = "Tick")]
pub struct GetTicksPeriod {
    pub code: String,
    pub date: String,
    pub end_date: String,
    pub skip: bool,
}

/// 获取因子值的 API，点击查看因子列表
/// 参数：
/// code: 单只股票代码
/// columns: 因子名称，因子名称，多个因子用逗号分隔
/// date: 开始日期
/// end_date: 结束日期
/// 返回：
/// date：日期
/// 查询因子值
/// 注：
/// 为保证数据的连续性，所有数据基于后复权计算
/// 为了防止单次返回数据时间过长，尽量较少查询的因子数和时间段
/// 如果第一次请求超时，尝试重试
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_factor_values")]
#[consume(format = "csv", type = "FactorValue")]
pub struct GetFactorValues {
    pub code: String,
    pub columns: String,
    pub date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FactorValue {
    pub date: String,
    pub cfo_to_ev: Option<f64>,
    pub net_profit_ratio: Option<f64>,
}

/// 模拟JQDataSDK的run_query方法
/// run_query api 是模拟了JQDataSDK run_query方法获取财务、宏观、期权等数据
/// 可查询的数据内容请查看JQData文档
/// 以查询上市公司分红送股（除权除息）数据为例：
/// 参数：
/// table: 要查询的数据库和表名，格式为 database + . + tablename 如finance.STK_XR_XD
/// columns: 所查字段，为空时则查询所有字段，多个字段中间用,分隔。如id,company_id，columns不能有空格等特殊字符
/// conditions: 查询条件，可以为空，格式为report_date#>=#2006-12-01&report_date#<=#2006-12-31，条件内部#号分隔，格式： column # 判断符 # value，多个条件使用&号分隔，表示and，conditions不能有空格等特殊字符
/// count: 查询条数，count为空时默认1条，最多查询1000条
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method(run_query)]
#[consume(format = "line")]
pub struct RunQuery {
    pub table: String,
    pub columns: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

/// 获取查询剩余条数
#[derive(Debug, Serialize, Deserialize, Jqdata)]
#[method("get_query_count")]
#[consume(format = "single", type = "i32")]
pub struct GetQueryCount {}
