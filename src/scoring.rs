use crate::indicators::Indicators;

#[derive(Debug, serde::Serialize)]
pub struct RuleHit{
    pub rule: String,
    pub points: i32,
}

#[derive(Debug, serde::Serialize)]
pub struct ScoreBreakdown{
    pub total: i32,
    pub hits: Vec<RuleHit>, //when rule condition are true
}

pub fn score_technical(ind: &Indicators) -> ScoreBreakdown {
    let mut hits: Vec<RuleHit> = Vec::new();
    if ind.last_close > ind.sma_200{
        hits.push(RuleHit{
            rule: "Close > SMA-200".to_string(),
            points: 20,
        });
    }
     if ind.last_close > ind.sma_50{
        hits.push(RuleHit{
            rule: "Close > SMA-50".to_string(),
            points: 10,
        });
    }
     if ind.sma_50 > ind.sma_200{
        hits.push(RuleHit{
            rule: "SMA-50 > SMA-200 (golden cross)".to_string(),
            points: 15,
        });
    }
     if ind.macd > ind.macd_signal{
        hits.push(RuleHit{
            rule: " MACD > signal (bullish)".to_string(),
            points: 15,
        });
    }
     if ind.macd_histogram > 0.0{
        hits.push(RuleHit{
            rule: " MACD > 0.0".to_string(),
            points: 10,
        });
    }
     if ind.rsi < 30.0{
        hits.push(RuleHit{
            rule: "oversold (potential bounce)".to_string(),
            points: 15,
        });
    }
     if ind.rsi > 70.0{
        hits.push(RuleHit{
            rule: "overbought (caution)".to_string(),
            points: -10,
        });
    }
     if ind.rsi >= 40.0 && ind.rsi <= 60.0{
        hits.push(RuleHit{
            rule: "RSI neutral healthy (40-60)".to_string(),
            points: 5,
        });
    }
     if ind.last_close < ind.bb_lower * 1.02 {
        hits.push(RuleHit{
            rule: "Close near BB lower".to_string(),
            points: 10,
        });
    }
     if ind.last_close > ind.bb_upper * 0.98 {
        hits.push(RuleHit{
            rule: "Close near BB upper (caution)".to_string(),
            points: -10,
        });
    }

    if ind.last_close < ind.sma_200 {
        hits.push(RuleHit {
            rule: "Close < SMA-200".to_string(),
            points: -20,
        });
    }

    if ind.last_close < ind.sma_50 {
        hits.push(RuleHit {
            rule: "Close < SMA-50".to_string(),
            points: -10,
        });
    }

    if ind.sma_50 < ind.sma_200 {
        hits.push(RuleHit {
            rule: "SMA-50 < SMA-200 (death cross)".to_string(),
            points: -15,
        });
    }

    if ind.macd < ind.macd_signal {
        hits.push(RuleHit {
            rule: "MACD < signal (bearish)".to_string(),
            points: -15,
        });
    }

    if ind.macd_histogram < 0.0 {
        hits.push(RuleHit {
            rule: "MACD histogram negative".to_string(),
            points: -10,
        });
    }


    let raw:i32 = hits.iter().map(|h| h.points).sum();
    let total = (50+raw).clamp(0,100);

    ScoreBreakdown{total, hits}
}
