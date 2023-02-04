    use std::collections::HashMap;
    use std::f64::consts::PI;
    use yahoo_finance_api::Quote;

    pub fn simple_moving_average(stock_data : &Vec<Quote>, days : i16) -> HashMap<&u64, f64> {
        let mut closed_sum : f64 = 0.;
        let mut date_sma_pairs = HashMap::new();
        for (index, record) in stock_data.iter().enumerate() {
            closed_sum += record.close;
            if index >= (days-1) as usize {
                date_sma_pairs.insert(&record.timestamp, closed_sum/days as f64);
                let address = index as i16 - days + 1;
                closed_sum -= stock_data[address as usize].close;
            }
       }
        date_sma_pairs
    }

    pub fn exponential_moving_average(stock_data : &Vec<Quote>, days : i16) -> HashMap<&u64, f64> {
        let mut date_ema_pair = HashMap::new();

        let alpha: f64 = 2. / (1. + days as f64);
        let mut last_ema =  stock_data.iter().take(days as usize).map(|x| x.close).sum::<f64>() / days as f64;
        date_ema_pair.insert(&stock_data[(days-1) as usize].timestamp, last_ema);
        for record in stock_data.iter().skip(days as usize) {
            let ema : f64 =  (record.close * alpha) + (last_ema * (1. - alpha));
            date_ema_pair.insert(&record.timestamp, ema);
            last_ema = ema;
        }
        date_ema_pair
    }

    pub fn mean(stock_data: &Vec<f64>) -> f64 {
        stock_data.iter().sum::<f64>() / stock_data.len() as f64
    }

    pub fn rsi(stock_data : &Vec<Quote>, period : i16) -> HashMap<&u64, f64> {
        //date_value_pair.push(DateClosePair {date: stock_data[0].timestamp, average_close : stock_data[0].close} );
        let mut count_to_period = 0;
        let mut gain_counter = 0;
        let mut lose_counter = 0;
        let mut sum_of_gain = 0.;
        let mut sum_of_lose = 0.;
        let mut rsi_points = HashMap::new();
        let mut previous_pos_average = 0.;
        let mut previous_neg_average = 0.;
        let mut first = false;
        for (index, record) in stock_data.iter().skip(1).enumerate() {
            let previous_prize = stock_data[index].close;
            let value_change = record.close - previous_prize;
            if count_to_period < period {
                //getting the percent of change between prices
                if value_change >= 0. {
                    gain_counter += 1;
                    sum_of_gain += value_change
                } else if value_change < 0. {
                    lose_counter += 1;
                    sum_of_lose += value_change;
                }
                count_to_period += 1;
            } else {
                if first {
                    if value_change < 0. {
                        previous_pos_average = (previous_pos_average * (period - 1) as f64 + 0.) / period as f64;
                        previous_neg_average = (previous_neg_average * (period - 1) as f64 + value_change.abs()) / period as f64;
                    } else {
                        previous_pos_average = (previous_pos_average * (period - 1) as f64 + value_change.abs()) / period as f64;
                        previous_neg_average = (previous_neg_average * (period - 1) as f64 + 0.) / period as f64;
                    }
                } else {
                    previous_pos_average = (sum_of_gain / gain_counter as f64) / period as f64;
                    previous_neg_average = (sum_of_lose.abs() / lose_counter as f64) / period as f64;
                    first = true;
                }
                let relative_strength = previous_pos_average / previous_neg_average;
                println!("Strength: {} ", relative_strength);
                let rsi_point = 100. - (100. / (1. + relative_strength));
                rsi_points.insert(&record.timestamp, rsi_point);
            }
        }
        rsi_points
    }

    pub fn macd(stock_data: &Vec<Quote>) -> HashMap<&u64,f64>{
        let ema12 = exponential_moving_average(stock_data, 12);
        let ema26 = exponential_moving_average(stock_data, 26);
        let mut macd_pairs: HashMap<&u64, f64> = HashMap::new();
        for record in ema12.iter(){
           if ema26.contains_key(record.0) {
                macd_pairs.insert(record.0,(record.1 - ema26.get(record.0).unwrap()).abs());
           }
            print!("MACDOCIE: {} ", (record.1 - ema26.get(record.0).unwrap()).abs());
        }
        macd_pairs
    }

    pub fn standard_deviation(variance: f64) -> f64 {
        variance.sqrt()
    }

    pub fn determine_trend(stock_data: &Vec<Quote>) -> HashMap<&u64, i8> {
        let mut direct_close_map = HashMap::new();
        for (index, record) in stock_data.iter().enumerate().skip(1) {
            if record.close > stock_data[index - 1 as usize].close {
                    direct_close_map.insert(&record.timestamp , 1);
            } else {
                direct_close_map.insert(&record.timestamp , 0 );
            }
        }
        direct_close_map
    }

    pub fn determine_volume_growth(stock_data: &Vec<Quote>) -> HashMap<u64, i8> {
        let mut direct_volume_map = HashMap::new();
        for (index, record) in stock_data.iter().enumerate().skip(1) {
            if record.volume > stock_data[index-1 as usize].volume {
                direct_volume_map.insert(record.timestamp , 1);
            } else {
                direct_volume_map.insert(record.timestamp , 0 );
            }
        }
        direct_volume_map
    }

    pub fn combine_values(macd : HashMap<&u64, f64>,ema : HashMap<&u64, f64>, sma : HashMap<&u64, f64>, direct_close: HashMap<&u64, i8> , data : &Vec<Quote> ) -> SignalsCombination {

        let mut dates_vec = Vec::new();
        let mut sma_vec = Vec::new();
        let mut ema_vec = Vec::new();
        let mut volume_vec = Vec::new();
        let mut direction_vec = Vec::new();
        let mut macd_vec = Vec::new();
        let volume_map = normalize_volume(data);
        for record in data {
            let date_in_float = record.timestamp;
            if ema.contains_key(&date_in_float) && sma.contains_key(&date_in_float) && macd.contains_key(&date_in_float) && direct_close.contains_key(&date_in_float){
                dates_vec.push(date_in_float);
                sma_vec.push(*sma.get(&date_in_float).unwrap());
                ema_vec.push(*ema.get(&date_in_float).unwrap());
                macd_vec.push(*macd.get(&date_in_float).unwrap());
                volume_vec.push(*volume_map.get(&date_in_float).unwrap());
                direction_vec.push(*direct_close.get(&date_in_float).unwrap() as f64);
            }
        }
        println!("SMA values: {:?}", sma_vec);
        println!("EMA values: {:?}", ema_vec);
        println!("MACD values: {:?}", macd_vec);
        println!("Volume values: {:?}", volume_vec);
        println!("Direction values: {:?}", direction_vec);
        SignalsCombination {
            dates: dates_vec,
            sma: sma_vec,
            ema: ema_vec,
            volume: volume_vec,
            direction: direction_vec,
            macd : macd_vec
        }
    }
    pub fn normalize_volume(data : &Vec<Quote>) -> HashMap<&u64, f64> {
        let mut normalized_volume = HashMap::new();
        //let volume_vec: Vec<u64> = data.into_iter().map(|x| x.volume).collect();
        // let min = volume_vec.iter().max().unwrap();
        // let max = volume_vec.iter().min().unwrap();
        for record in data {
            //let normalized = (record.volume as f64 - *min as f64)  / (*max as f64 - *min as f64);
            normalized_volume.insert(&record.timestamp, (record.volume / 1000) as f64);
        }
        normalized_volume
    }

    pub fn group_under_trend(train_data : &SignalsCombination) -> BinarySplit {
        let mut splitted_data = BinarySplit {
            positive: SignalsCombination {
                dates: Vec::new(),
                sma: Vec::new(),
                ema: Vec::new(),
                volume: Vec::new(),
                direction: Vec::new(),
                macd : Vec::new()
            },
            negative: SignalsCombination {
                dates: Vec::new(),
                sma: Vec::new(),
                ema: Vec::new(),
                volume: Vec::new(),
                direction: Vec::new(),
                macd : Vec::new()
            }
        };

        for (index, record) in  train_data.direction.iter().enumerate() {
            if *record  == 1. as f64 {
                splitted_data.positive.sma.push(train_data.sma[index]);
                splitted_data.positive.ema.push(train_data.ema[index]);
                splitted_data.positive.macd.push(train_data.macd[index]);
                splitted_data.positive.volume.push(train_data.volume[index]);
                splitted_data.positive.dates.push(train_data.dates[index]);
            } else {
                splitted_data.negative.sma.push(train_data.sma[index]);
                splitted_data.negative.ema.push(train_data.ema[index]);
                splitted_data.negative.macd.push(train_data.macd[index]);
                splitted_data.negative.volume.push(train_data.volume[index]);
                splitted_data.negative.dates.push(train_data.dates[index]);
                println!("Its negative");

            }
        }
        splitted_data
    }

    pub fn mean_and_std(splitted_data : &BinarySplit) -> (MeanStdPairs, MeanStdPairs) {
        //means for positive outcome
       let positive =  MeanStdPairs {
            ema_pair: get_mean_and_std(&splitted_data.positive.ema),
            sma_pair: get_mean_and_std(&splitted_data.positive.sma),
            macd_pair: get_mean_and_std(&splitted_data.positive.macd),
            volume_pair: get_mean_and_std(&splitted_data.positive.volume)
        };

        //means for negative outcome
        let negative =  MeanStdPairs {
            ema_pair: get_mean_and_std(&splitted_data.negative.ema),
            sma_pair: get_mean_and_std(&splitted_data.negative.sma),
            macd_pair: get_mean_and_std(&splitted_data.negative.macd),
            volume_pair: get_mean_and_std(&splitted_data.negative.volume)
        };
        println!("Positive ema pair {:?}", positive.ema_pair);
        println!("Positive sma pair {:?}", positive.sma_pair);
        println!("Positive macd pair {:?}", positive.macd_pair);
        (positive, negative)
    }

    pub fn gaussian_probability(mean_std: (f64, f64), value : f64) -> f64 {
        // w przypadku obliczenia funkcji gęstości gaussa wykorzystujemy stałą Eulera i potęgujemy ją, za proces ten odpowiedzialna jest funkcja f64::exp
        //parametrami fu
        println!("Mean and std: {:?}", mean_std);
        let exponent =  f64::exp(-((f64::powi(value - mean_std.0,2))/ (2. * f64::powi(mean_std.1, 2))));
        println!("Exponent: {}", &exponent);
        println!("Czesc Gaussa: {}", (1./ (f64::sqrt(2. * PI) * mean_std.1)));
        let gauss: f64 = (1./ (f64::sqrt(2. * PI) * mean_std.1)) * exponent;
        gauss
    }

    fn get_mean_and_std(attribute: &Vec<f64>) -> (f64,f64) {
        let mean = mean(attribute);
        println!("MEAN: {}", &mean);
        let std = standard_deviation(variance(attribute, &mean));
        println!("STANDARD DEVIATION: {}", &std);
        (mean, std)
    }

    pub fn variance(attribute_data : &Vec<f64>, mean : &f64) -> f64 {
        let mut sum = 0.;
        for record in attribute_data {
           sum = sum + (record - mean).powi(2);
        }
       sum / attribute_data.len() as f64 //zwrocenie wariancji
    }

   pub fn split_data(combined_signals : SignalsCombination, ratio : f64 )  -> SplitedComposition {
    let data_length = combined_signals.dates.len();
    let mut train_length = data_length as f64 *  ratio;
    train_length = train_length.ceil();
       let train_length = train_length as usize;
      let splitted_dates = combined_signals.dates.split_at(train_length);
      let splitted_sma = combined_signals.sma.split_at(train_length);
      let splitted_ema = combined_signals.ema.split_at(train_length);
      let splitted_macd = combined_signals.macd.split_at(train_length);
      let splitted_volume = combined_signals.volume.split_at(train_length);
      let splitted_direction = combined_signals.direction.split_at(train_length);
        println!("Splitted volume: {:?}", splitted_volume);
       SplitedComposition {
           training: SignalsCombination {
               dates: splitted_dates.0.to_vec(),
               sma: splitted_sma.0.to_vec(),
               ema: splitted_ema.0.to_vec(),
               volume: splitted_volume.0.to_vec(),
               direction: splitted_direction.0.to_vec(),
               macd : splitted_macd.0.to_vec()
           },
           testing: SignalsCombination {
               dates: splitted_dates.1.to_vec(),
               sma: splitted_sma.1.to_vec(),
               ema: splitted_ema.1.to_vec(),
               volume: splitted_volume.1.to_vec(),
               direction: splitted_direction.1.to_vec(),
               macd : splitted_macd.1.to_vec()
           }
       }
}

    pub fn naive_bayes(data : Vec<Quote>, days : i16, ratio : f64) -> Vec<(f64, i64)> {
        let sma = simple_moving_average(&data,days);
        let ema = exponential_moving_average(&data,days);
        let macd = macd(&data);
        //let volume = determine_volume_growth(&data);
        let close = determine_trend(&data);

        let mut combination = combine_values(macd,ema, sma,close , &data);
        let mut splitted_data = split_data(combination, ratio);
        let mut trained_divided = group_under_trend(&splitted_data.training);
        //pairs stores 2 objects for both negative and positive values divided into indicator data
        let mut pairs = mean_and_std(&trained_divided); // 0 from pair is positive, 1 is negative
        let predictions = splitted_data.testing.volume.len() as i64;

        let positives_count =  trained_divided.positive.volume.len() as f64;
        let negatives_count =  trained_divided.negative.volume.len() as f64;
        let positives_ratio = positives_count / (negatives_count + positives_count);
        let negatives_ratio = negatives_count / (negatives_count + positives_count);
        let mut test_with_pred : Vec<(f64, i64)> = Vec::new();
        for index in 0.. predictions - 1 {
            //dane z wynikiem pozytywnym
            let positive_data = count_gauss_probability(&mut splitted_data.testing, index as usize, &pairs.0);
            println!("Positive data probability: {}", positive_data);

            //dane z wynikiem negatywnym
            let negative_data = count_gauss_probability(&mut splitted_data.testing, index as usize, &pairs.1);
            println!("Negative data probability: {}", positive_data);

            if (positives_ratio * positive_data) > (negatives_ratio * negative_data) {
                test_with_pred.push( (splitted_data.testing.direction[index as usize], 1));
            } else {
                test_with_pred.push( (splitted_data.testing.direction[index as usize], 0));

            }
        }
        test_with_pred
    }

    fn count_gauss_probability(testing: &mut SignalsCombination, index: usize, data: &MeanStdPairs) -> f64 {
        let macd_data = data.macd_pair;
        println!("MACD: {:?}", macd_data);
        let volume_data = data.volume_pair;
        println!("Volume: {:?}", volume_data);

        let sma_data = data.sma_pair;
        println!("SMA: {:?}", sma_data);

        let ema_data = data.ema_pair;
        println!("EMA: {:?}", ema_data);

        let record = testing;

        let macd_gauss = gaussian_probability(macd_data, record.macd[index]);
        println!("Jestem przed liczeniem volume");
        let volume_gauss = gaussian_probability(volume_data, record.volume[index]);
        let sma_gauss = gaussian_probability(sma_data, record.sma[index]);
        let ema_gauss = gaussian_probability(ema_data, record.ema[index]);
        let probability = macd_gauss * volume_gauss * sma_gauss * ema_gauss;
        println!("MACD: {}, Volume: {}, sma: {}, ema: {}", &macd_gauss, &volume_gauss, &sma_gauss, &ema_gauss);
        probability
    }

    pub struct DirectVolume {
        pub direction : i8,
        pub volume : Option<u64>
    }

    pub struct DateValuePair {
        pub date: i64,
        pub value: f64
    }

    pub struct IndicatorCombination {
        pub dates : Vec<i64>,
        pub sma : Vec<i8>,
        pub ema : Vec<i8>,
        pub volume : Vec<i8>,
        pub direction : Vec<i8>,
        pub macd : Vec<i8>
    }
    pub struct SignalsCombination {
        pub dates : Vec<u64>,
        pub sma : Vec<f64>,
        pub ema : Vec<f64>,
        pub volume : Vec<f64>,
        pub direction : Vec<f64>,
        pub macd : Vec<f64>
    }
    pub struct SplitedComposition {
        pub training : SignalsCombination,
        pub testing : SignalsCombination
    }
    pub struct BinarySplit {
        pub positive: SignalsCombination,
        pub negative : SignalsCombination
    }
    pub struct MeanStdPairs {
        pub  ema_pair : (f64, f64),
        pub sma_pair : (f64, f64),
        pub macd_pair : (f64, f64),
        pub volume_pair : (f64, f64),
    }

