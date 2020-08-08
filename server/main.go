package main

import (
	"fmt"
	"log"
	"math/rand"
	"net/http"
)

// Ticker ....
type Ticker struct {
	name         string
	initialPrice float64
	change       float64
}

var tickers map[string]Ticker = make(map[string]Ticker)

func getTicker(t *Ticker) string {
	var price = t.initialPrice + t.change
	return fmt.Sprintf(`{"quoteSummary": {"result": [{"summaryDetail": {"maxAge": 1,"priceHint": {"raw": 2,"fmt": "2","longFmt": "2"},"previousClose": {"raw": 1482.76,"fmt": "1,482.76"},"open": {"raw": 1486.71,"fmt": "1,486.71"},"dayLow": {"raw": 1464.03,"fmt": "1,464.03"},"dayHigh": {"raw": 1492.352,"fmt": "1,492.35"},"regularMarketPreviousClose": {"raw": 1482.76,"fmt": "1,482.76"},"regularMarketOpen": {"raw": 1486.71,"fmt": "1,486.71"},"regularMarketDayLow": {"raw": 1464.03,"fmt": "1,464.03"},"regularMarketDayHigh": {"raw": 1492.352,"fmt": "1,492.35"},"dividendRate": {},"dividendYield": {},"exDividendDate": {},"payoutRatio": {"raw": 0.0,"fmt": "0.00%%"},"fiveYearAvgDividendYield": {},"beta": {"raw": 1.064985,"fmt": "1.06"},"trailingPE": {"raw": 32.385914,"fmt": "32.39"},"forwardPE": {"raw": 26.201317,"fmt": "26.20"},"volume": {"raw": 1798566,"fmt": "1.8M","longFmt": "1,798,566"},"regularMarketVolume": {"raw": 1798566,"fmt": "1.8M","longFmt": "1,798,566"},"averageVolume": {"raw": 1842932,"fmt": "1.84M","longFmt": "1,842,932"},"averageVolume10days": {"raw": 2114683,"fmt": "2.11M","longFmt": "2,114,683"},"averageDailyVolume10Day": {"raw": 2114683,"fmt": "2.11M","longFmt": "2,114,683"},"bid": {"raw": 1470.05,"fmt": "1,470.05"},"ask": {"raw": 1472.0,"fmt": "1,472.00"},"bidSize": {"raw": 1100,"fmt": "1.1k","longFmt": "1,100"},"askSize": {"raw": 1000,"fmt": "1k","longFmt": "1,000"},"marketCap": {"raw": 999330611200,"fmt": "999.33B","longFmt": "999,330,611,200"},"yield": {},"ytdReturn": {},"totalAssets": {},"expireDate": {},"strikePrice": {},"openInterest": {},"fiftyTwoWeekLow": {"raw": 1008.87,"fmt": "1,008.87"},"fiftyTwoWeekHigh": {"raw": 1587.05,"fmt": "1,587.05"},"priceToSalesTrailing12Months": {"raw": 6.018976,"fmt": "6.02"},"fiftyDayAverage": {"raw": 1484.6989,"fmt": "1,484.70"},"twoHundredDayAverage": {"raw": 1374.1932,"fmt": "1,374.19"},"trailingAnnualDividendRate": {},"trailingAnnualDividendYield": {},"navPrice": {},"currency": "USD","fromCurrency": null,"toCurrency": null,"lastMarket": null,"volume24Hr": {},"volumeAllCurrencies": {},"circulatingSupply": {},"algorithm": null,"maxSupply": {},"startDate": {},"tradeable": false},"price": {"maxAge": 1,"preMarketChangePercent": {"raw": 9.50952E-4,"fmt": "0.10%%"},"preMarketChange": {"raw": 1.41003,"fmt": "1.41"},"preMarketTime": 1596547799,"preMarketPrice": {"raw": 1484.17,"fmt": "1,484.17"},"preMarketSource": "FREE_REALTIME","postMarketChangePercent": {"raw": -0.002572483,"fmt": "-0.26%%"},"postMarketChange": {"raw": -3.790039,"fmt": "-3.79"},"postMarketTime": 1596572662,"postMarketPrice": {"raw": 1469.51,"fmt": "1,469.51"},"postMarketSource": "DELAYED","regularMarketChangePercent": {"raw": -0.0063799676,"fmt": "-0.64%%"},"regularMarketChange": {"raw": %f,"fmt": "%f"},"regularMarketTime": 1596571201,"priceHint": {"raw": 2,"fmt": "2","longFmt": "2"},"regularMarketPrice": {"raw": %f,"fmt": "%f"},"regularMarketDayHigh": {"raw": 1492.352,"fmt": "1,492.35"},"regularMarketDayLow": {"raw": 1464.03,"fmt": "1,464.03"},"regularMarketVolume": {"raw": 1798566,"fmt": "1.80M","longFmt": "1,798,566.00"},"averageDailyVolume10Day": {"raw": 2114683,"fmt": "2.11M","longFmt": "2,114,683"},"averageDailyVolume3Month": {"raw": 1842932,"fmt": "1.84M","longFmt": "1,842,932"},"regularMarketPreviousClose": {"raw": 1482.76,"fmt": "1,482.76"},"regularMarketSource": "FREE_REALTIME","regularMarketOpen": {"raw": 1486.71,"fmt": "1,486.71"},"strikePrice": {},"openInterest": {},"exchange": "NMS","exchangeName": "NasdaqGS","exchangeDataDelayedBy": 0,"marketState": "POST","quoteType": "EQUITY","symbol": "%s","underlyingSymbol": null,"shortName": "Alphabet Inc.","longName": "Alphabet Inc.","currency": "USD","quoteSourceName": "Delayed Quote","currencySymbol": "$","fromCurrency": null,"toCurrency": null,"lastMarket": null,"volume24Hr": {},"volumeAllCurrencies": {},"circulatingSupply": {},"marketCap": {"raw": 999330611200,"fmt": "999.33B","longFmt": "999,330,611,200.00"}}}],"error": null}}`, t.change, t.change, price, price, t.name)
}

func handler(w http.ResponseWriter, r *http.Request) {
	var ticker = r.URL.Path
	if t, ok := tickers[ticker]; ok {
		t.change += (rand.Float64() - 0.5) * (float64)(rand.Intn(4-1)+1)
		tickers[ticker] = t
	} else {
		t = Ticker{
			name:         ticker,
			initialPrice: (float64)(rand.Intn(1900-300) + 300),
			change:       (rand.Float64() - 0.5),
		}
		fmt.Printf("initial price for %s is %f\n", t.name, t.initialPrice)
		tickers[ticker] = t
	}
	var t = tickers[ticker]
	w.Write([]byte(getTicker(&t)))
}

func main() {
	http.HandleFunc("/", handler)
	log.Fatal(http.ListenAndServe(":8080", nil))
}
