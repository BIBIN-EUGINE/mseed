//! Port of [libmseed
//! `lm_pack.c`](https://github.com/EarthScope/libmseed/blob/main/example/lm_pack.c) example.
//!
//! For further information on how to use this example program, simply invoke:
//!
//! ```sh
//! cargo run --example pack -- --help
//! ```

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use mseed::{MSControlFlags, MSDataEncoding, PackInfo};

/// An expanding sinusoid of 500 samples.
///
/// When truncated to integers:
/// - Signed differences in samples needing all bit lengths 3-31s.
/// - ONLY the last difference requires 31 bits, all others are <= 30
/// -  Contain sequences of differences matching all Steim 1 and 2 encoding possibilities
///
/// Values 1 through up to 220 do not require more than 16-bits.
static SINE: &[f32] = &[
    0.000000,
    6.109208,
    10.246826,
    10.609957,
    6.764728,
    -0.075704,
    -7.409461,
    -12.346208,
    -12.731430,
    -8.062958,
    0.182060,
    8.985442,
    14.875067,
    15.276420,
    9.609196,
    -0.328370,
    -10.895428,
    -17.921131,
    -18.329336,
    -11.450576,
    0.526448,
    13.209973,
    21.590023,
    21.991385,
    13.643140,
    -0.791247,
    -16.014492,
    -26.008907,
    -26.383901,
    -16.253504,
    1.141655,
    19.412378,
    31.330871,
    31.652348,
    19.360848,
    -1.601465,
    -23.528777,
    -37.740204,
    -37.971107,
    -23.059260,
    2.200591,
    28.515156,
    45.458753,
    45.549217,
    27.460529,
    -2.976569,
    -34.554817,
    -54.753559,
    -54.637244,
    -32.697448,
    3.976416,
    41.869576,
    65.946052,
    65.535525,
    38.927729,
    -5.258928,
    -50.727832,
    -79.423116,
    -78.604029,
    -46.338644,
    6.897516,
    61.454325,
    95.650397,
    94.274177,
    55.152523,
    -8.983716,
    -74.441929,
    -115.188314,
    -113.063003,
    -65.633259,
    11.631511,
    90.165916,
    138.711322,
    135.590105,
    78.094013,
    -14.982665,
    -109.201193,
    -167.031082,
    -162.597954,
    -92.906331,
    19.213290,
    132.243134,
    201.124338,
    194.976222,
    110.510935,
    -24.541940,
    -160.132765,
    -242.166440,
    -233.790899,
    -131.430478,
    31.239561,
    193.887193,
    291.571675,
    280.319178,
    156.284630,
    -39.641741,
    -234.736391,
    -351.041770,
    -336.091216,
    -185.807912,
    50.163757,
    284.167632,
    422.624233,
    402.940140,
    220.870763,
    -63.319072,
    -343.979194,
    -508.782510,
    -483.061913,
    -262.504429,
    79.742042,
    416.345244,
    612.480366,
    579.087014,
    311.930343,
    -100.215795,
    -503.894220,
    -737.283353,
    -694.166245,
    -370.594813,
    125.706429,
    609.803540,
    887.480834,
    832.073438,
    440.209940,
    -157.404955,
    -737.913996,
    -1068.232708,
    -997.328409,
    -522.801872,
    196.778712,
    892.867950,
    1285.745847,
    1195.344115,
    620.767701,
    -245.634371,
    -1080.276240,
    -1547.486227,
    -1432.602775,
    -736.942489,
    306.195102,
    1306.919790,
    1862.433994,
    1716.866654,
    874.678227,
    -381.195062,
    -1580.993078,
    -2241.390126,
    -2057.430298,
    -1037.936774,
    473.995052,
    1912.398180,
    2697.345105,
    2465.422369,
    1231.399220,
    -588.724017,
    -2313.099844,
    -3245.922143,
    -2954.166800,
    -1460.594493,
    730.452133,
    2797.554253,
    3905.910007,
    3539.614901,
    1732.050527,
    -905.402443,
    -3383.226729,
    -4699.903513,
    -4240.862333,
    -2053.471832,
    1121.209554,
    4091.216806,
    5655.073452,
    5080.767553,
    2433.947965,
    -1387.235765,
    -4947.012887,
    -6804.092030,
    -6086.691631,
    -2884.198121,
    1714.957253,
    5981.403297,
    8186.245216,
    7291.383170,
    3416.857907,
    -2118.435721,
    -7231.576094,
    -9848.769674,
    -8734.036728,
    -4046.815355,
    2614.894255,
    8742.446660,
    11848.459577,
    10461.558685,
    4791.604321,
    -3225.420222,
    -10568.260176,
    -14253.597692,
    -12530.081077,
    -5671.864737,
    3975.823020,
    12774.525771,
    17146.276092,
    15006.771888,
    6711.880612,
    -4897.680529,
    -15440.350897,
    -20625.184991,
    -17971.999652,
    -7940.208402,
    6029.615451,
    18661.258563,
    24808.964001,
    21521.921578,
    9390.410233,
    -7418.851697,
    -22552.587165,
    -29840.229074,
    -25771.577789,
    -11101.908663,
    9123.111793,
    27253.593143,
    35890.411153,
    30858.590420,
    13120.982075,
    -11212.929535,
    -32932.401520,
    -43165.569941,
    -36947.585456,
    -15501.922592,
    13774.468128,
    39791.979238,
    51913.378983,
    44235.478131,
    18308.381438,
    -16912.953578,
    -48077.342225,
    -62431.517747,
    -52957.790070,
    -21614.930138,
    20756.856764,
    58084.250586,
    75077.753677,
    63396.198979,
    25508.869716,
    -25462.986446,
    -70169.698634,
    -90282.054065,
    -75887.560674,
    -30092.324242,
    31222.690356,
    84764.569627,
    108561.135815,
    90834.689747,
    35484.659605,
    -38269.404043,
    -102388.901147,
    -130535.943080,
    -108719.240618,
    -41825.273267,
    46887.838631,
    123670.298781,
    156952.641109,
    130117.096987,
    49276.805933,
    -57425.161274,
    -149366.146307,
    -188707.832667,
    -155716.756615,
    -58028.831480,
    70304.598043,
    180390.393831,
    226878.845103,
    186341.292651,
    68302.086980,
    -86041.981200,
    -217845.865904,
    -272760.106197,
    -222974.585037,
    -80353.310040,
    105265.874749,
    263063.225186,
    327906.831053,
    266792.649626,
    94480.755768,
    -128742.047924,
    -317647.960457,
    -394187.487304,
    -319201.052420,
    -111030.470003,
    157403.231090,
    383537.048821,
    473846.799900,
    381879.586964,
    130403.398591,
    -192385.288422,
    -463067.280664,
    -569581.409624,
    -456835.620147,
    -153063.413757,
    235071.184227,
    559057.643966,
    684630.722900,
    546467.782541,
    179546.337116,
    -287144.413901,
    -674908.656359,
    -822885.998573,
    -653642.002301,
    -210470.033463,
    350653.927224,
    814722.125712,
    989021.327043,
    781782.266485,
    246545.638548,
    -428093.004245,
    -983445.533844,
    -1188650.888674,
    -934978.952245,
    -288589.965744,
    522495.068481,
    1187046.097893,
    1428517.756157,
    1118118.116795,
    337539.108116,
    -637550.058001,
    -1432720.599829,
    -1716720.558561,
    -1337035.786123,
    -394463.210730,
    777745.745791,
    1729148.322568,
    2062985.588205,
    1598702.057951,
    460582.328770,
    -948539.335064,
    -2086795.934450,
    -2478993.447111,
    -1911440.758239,
    -537283.204837,
    1156565.787585,
    2518284.974613,
    2978771.147961,
    2285191.490698,
    626136.686773,
    -1409890.715316,
    -3038834.772892,
    -3579162.765425,
    -2731822.228828,
    -728915.356799,
    1718317.328563,
    3666796.264850,
    4300394.349825,
    3265502.159830,
    847610.742696,
    -2093758.948552,
    -4424295.326464,
    -5166751.952850,
    -3903146.346575,
    -984449.218292,
    2550691.033230,
    5338008.063395,
    6207395.378498,
    4664945.984099,
    1141905.356234,
    -3106699.622013,
    -6440095.078674,
    -7457334.785992,
    -5575000.657492,
    -1322711.048944,
    3783146.686785,
    7769327.268660,
    8958602.684417,
    6662072.138144,
    1529858.136359,
    -4605977.214406,
    -9372442.351616,
    -10761660.350202,
    -7960482.979376,
    -1766591.537123,
    5606698.099533,
    11305779.346088,
    12927085.482769,
    9511187.602536,
    2036388.930375,
    -6823565.288237,
    -13637247.864482,
    -15527597.247686,
    -11363048.833756,
    -2342921.824577,
    8303023.316062,
    16448700.704063,
    18650486.048610,
    13574359.117581,
    2689991.311114,
    -10101450.710498,
    -19838792.204081,
    -22400528.788313,
    -16214653.084049,
    -3081429.850680,
    12287276.018778,
    23926421.676653,
    26903486.467518,
    19366867.002831,
    3520957.976311,
    -14943542.889821,
    -28854881.491245,
    -32310300.258257,
    -23129911.185536,
    -4011981.690190,
    18171019.183242,
    34796853.797034,
    38802125.311214,
    27621733.908111,
    4557312.422997,
    -22091965.102165,
    -41960429.245394,
    -46596369.274924,
    -32982970.290335,
    -5158786.517383,
    26854699.579994,
    50596356.438136,
    55953935.730766,
    39381287.228525,
    5816755.047390,
    -32639133.476996,
    -61006773.393892,
    -67187912.573574,
    -47016556.451947,
    -6529407.091211,
    39663473.627686,
    73555723.560159,
    80673993.102082,
    56127012.680548,
    7291879.963058,
    -48192344.715084,
    -88681820.564997,
    -96862974.793882,
    -66996583.433046,
    -8095097.920129,
    58546627.892657,
    106913500.120816,
    116295749.303753,
    79963612.136585,
    8924265.924556,
    -71115377.915425,
    -128887386.814358,
    -139621279.391140,
    -95431237.847443,
    -9756926.457174,
    86370256.557591,
    155370411.011265,
    167618156.945886,
    113879744.320966,
    10560464.285645,
    -104883012.041142,
    -187286440.462195,
    -201220454.267134,
    -135881249.803956,
    -11288915.420918,
    127346645.407973,
    225748346.861610,
    241548722.121826,
    162117178.463787,
    11878900.943144,
    -154601039.263161,
    -272096614.921471,
    -289947157.492990,
    -193399036.824020,
    -12244462.331299,
    187663986.962885,
    327945826.905572,
    348028166.863781,
    230693116.321629,
    12270520.408649,
    -227768756.995135,
    -395240626.753420,
    -417725794.004062,
    -275149858.943091,
    -11804612.586023,
    276409565.108355,
    476323094.198962,
    501359772.474058,
    328138760.150134,
    10646479.758475,
    -335396614.264246,
    -574013851.836865,
    -601712311.942546,
    -391289845.886545,
    -8534971.317913,
    406922710.094078,
    691709700.348455,
    722120145.317499,
    466542952.987464,
    5131609.783276,
    -493643879.751773,
    -833501145.234545,
    -866584864.231526,
    -556206270.243475,
    0.00000,
];

static TEXT: &str = "I've seen things you people wouldn't believe. Attack ships on fire off the
shoulder of Orion. I watched C-beams glitter in the dark near the Tannhäuser Gate. All those
moments will be lost in time, like tears...in...rain. Time to die.";

#[derive(Parser, Debug)]
#[command(author, version)]
#[command(about = "Packs static, test data into miniSEED", long_about = None)]
struct Args {
    /// Specify data encoding format
    #[arg(value_enum)]
    #[arg(short = 'e', default_value_t = DataEncoding::Steim2)]
    encoding: DataEncoding,

    /// Specify maximum record length.
    #[arg(short = 'r', long, value_name = "BYTES", default_value_t = 4096)]
    rec_len: i32,

    /// Specify miniSEED version format.
    #[arg(short = 'F', long, value_name = "FORMAT", default_value_t = 3)]
    #[arg(value_parser = clap::value_parser!(u8).range(2..4))]
    format: u8,

    /// Path to output file.
    #[arg(value_name = "FILE")]
    out_file: PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum DataEncoding {
    /// Text encoding (UTF-8)
    Text,
    /// 16-bit integer encoding
    Integer16,
    /// 32-bit integer encoding
    Integer32,
    /// 32-bit floating point encoding (IEEE)
    Float32,
    /// 64-bit floating point encoding (IEEE)
    Float64,
    /// Steim-1 compressed integer encoding
    Steim1,
    /// Steim-2 compressed integer encoding
    Steim2,
}

fn main() {
    let args = Args::parse();

    let mut sine_data_i32: Vec<i32> = SINE.iter().map(|n| *n as i32).collect();
    let mut sine_data_f32: Vec<f32> = SINE.iter().map(|n| *n).collect();
    let mut sine_data_f64: Vec<f64> = SINE.iter().map(|n| *n as f64).collect();
    let mut text = TEXT.as_bytes().to_vec();

    let mut pack_info = PackInfo::with_sample_rate("FDSN:XX_TEST__X_Y_Z", 1.0).unwrap();
    pack_info.rec_len = args.rec_len;
    pack_info.format_version = args.format;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(args.out_file)
        .unwrap();
    let mut writer = BufWriter::new(file);

    let record_handler = move |rec: &[u8]| {
        let _ = writer.write(rec);
    };

    let start_time = OffsetDateTime::parse("2012-01-01T00:00:00Z", &Iso8601::DEFAULT).unwrap();
    let flags = MSControlFlags::MSF_FLUSHDATA;

    match args.encoding {
        DataEncoding::Text => {
            pack_info.encoding = MSDataEncoding::Text;
            mseed::pack_raw(&mut text, &start_time, record_handler, &pack_info, flags)
        }
        DataEncoding::Integer16 => {
            pack_info.encoding = MSDataEncoding::Integer16;
            // The first 220 samples can be represented in 16-bits
            mseed::pack_raw(
                &mut sine_data_i32[..220],
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
        DataEncoding::Integer32 => {
            pack_info.encoding = MSDataEncoding::Integer32;
            mseed::pack_raw(
                &mut sine_data_i32,
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
        DataEncoding::Float32 => {
            pack_info.encoding = MSDataEncoding::Float32;
            mseed::pack_raw(
                &mut sine_data_f32,
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
        DataEncoding::Float64 => {
            pack_info.encoding = MSDataEncoding::Float64;
            mseed::pack_raw(
                &mut sine_data_f64,
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
        DataEncoding::Steim1 => {
            pack_info.encoding = MSDataEncoding::Steim1;
            mseed::pack_raw(
                &mut sine_data_i32,
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
        DataEncoding::Steim2 => {
            pack_info.encoding = MSDataEncoding::Steim2;
            // Steim-2 can represent all but the last difference
            mseed::pack_raw(
                &mut sine_data_i32[..499],
                &start_time,
                record_handler,
                &pack_info,
                flags,
            )
        }
    }
    .unwrap();
}
