use crate::indicators::Indicators;
use crate::models::Fundamentals;
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

pub fn score_fundamental(f: &Fundamentals) -> ScoreBreakdown {
    let mut hits: Vec<RuleHit> = Vec::new();

    // --- VALUATION ---
    if let Some(pe) = f.pe_ratio {
        if pe > 0.0 && pe < 15.0 {
            hits.push(RuleHit { rule: "P/E < 15 (cheap)".into(), points: 15 });
        } else if pe >= 15.0 && pe <= 25.0 {
            hits.push(RuleHit { rule: "P/E 15-25 (fair)".into(), points: 5 });
        } else if pe > 30.0 {
            hits.push(RuleHit { rule: "P/E > 30 (expensive)".into(), points: -10 });
        }
        // negative P/E = company losing money
        if pe < 0.0 {
            hits.push(RuleHit { rule: "P/E negative (no earnings)".into(), points: -15 });
        }
    }

    if let Some(peg) = f.peg_ratio {
        if peg > 0.0 && peg < 1.0 {
            hits.push(RuleHit { rule: "PEG < 1 (cheap vs growth)".into(), points: 15 });
        } else if peg >= 1.0 && peg <= 2.0 {
            hits.push(RuleHit { rule: "PEG 1-2 (fair vs growth)".into(), points: 5 });
        } else if peg > 2.0 {
            hits.push(RuleHit { rule: "PEG > 3 (expensive vs growth)".into(), points: -10 });
        }
    }

    // --- PROFITABILITY ---
    if let Some(margin) = f.profit_margin {
        if margin > 0.20 {
            hits.push(RuleHit { rule: "Profit margin > 20%".into(), points: 10 });
        } else if margin > 0.10 {
            hits.push(RuleHit { rule: "Profit margin 10-20%".into(), points: 5 });
        } else if margin < 0.0 {
            hits.push(RuleHit { rule: "Negative profit margin".into(), points: -15 });
        }
    }

    if let Some(roe) = f.roe {

        let capped = roe.min(0.50);
        if capped > 0.20 {
            hits.push(RuleHit { rule: "ROE > 20%".into(), points: 10 });
        } else if capped > 0.10 {
            hits.push(RuleHit { rule: "ROE 10-20%".into(), points: 5 });
        } else if roe < 0.0 {
            hits.push(RuleHit { rule: "ROE negative".into(), points: -10 });
        }
    }

    // --- LEVERAGE ---
    if let Some(de) = f.debt_to_equity {
        if de >= 0.0 && de < 0.5 {
            hits.push(RuleHit { rule: "Debt/Equity < 0.5 (low leverage)".into(), points: 10 });
        } else if de > 2.0 {
            hits.push(RuleHit { rule: "Debt/Equity > 2.0 (high leverage)".into(), points: -10 });
        }
    }

    // --- GROWTH ---
    if let Some(rg) = f.quarterly_revenue_growth {
        if rg > 0.20 {
            hits.push(RuleHit { rule: "Revenue growth > 20% YoY".into(), points: 10 });
        } else if rg > 0.05 {
            hits.push(RuleHit { rule: "Revenue growth 5-20% YoY".into(), points: 5 });
        } else if rg < 0.0 {
            hits.push(RuleHit { rule: "Revenue declining YoY".into(), points: -10 });
        }
    }

    if let Some(eg) = f.quarterly_earnings_growth {
        if eg > 0.20 {
            hits.push(RuleHit { rule: "Earnings growth > 20% YoY".into(), points: 10 });
        } else if eg > 0.05 {
            hits.push(RuleHit { rule: "Earnings growth 5-20% YoY".into(), points: 5 });
        } else if eg < 0.0 {
            hits.push(RuleHit { rule: "Earnings declining YoY".into(), points: -10 });
        }
    }

    // --- DIVIDEND (small positive only — not central to scoring) ---
    if let Some(dy) = f.dividend_yield {
        if dy > 0.03 {
            hits.push(RuleHit { rule: "Dividend yield > 3%".into(), points: 5 });
        }
    }

    let raw: i32 = hits.iter().map(|h| h.points).sum();
    let total = (50 + raw).clamp(0, 100);

    ScoreBreakdown { total, hits }
}

#[derive(Debug, serde::Serialize)]
pub struct CombinedScore {
    pub total: i32,
    pub technical: ScoreBreakdown,
    pub fundamental: Option<ScoreBreakdown>,
}

pub fn score_combined(
    technical: ScoreBreakdown,
    fundamental: Option<ScoreBreakdown>,
) -> CombinedScore {
    let total = match &fundamental {
        Some(f) => {
            // 60% technical, 40% fundamental
            let weighted = (technical.total as f64 * 0.6) + (f.total as f64 * 0.4);
            weighted.round() as i32
        }
        None => technical.total,
    };

    CombinedScore { total, technical, fundamental }
}