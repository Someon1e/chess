use crate::board::game_state::CastlingRights;
use crate::board::piece::Piece;
use crate::board::square::Square;
use crate::consume_bit_board;

use super::Board;

/// Filled with random integers.
#[derive(Debug)]
pub struct ZobristRandoms {
    /// Random integers for every piece for every square.
    pub piece_arrays: [[u64; 64]; 12],

    /// Random integer.
    pub side_to_move: u64,

    /// Random integers for every file.
    pub en_passant_square_file: [u64; 8],

    /// Random integers for every castling rights state.
    pub castling_rights: [u64; 16],
}

#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
const ZOBRIST_RANDOMS: ZobristRandoms = ZobristRandoms { piece_arrays: [[13101417551045075907, 10137645642488790847, 18342714896108701504, 17310758865097165602, 17738235767275252540, 11541341906267115112, 5304675379255936998, 10913644484567115956, 10691177763299342419, 3771262636225135816, 2702651789367557300, 12597713371813889959, 10909436146348530443, 9173447860719345744, 9123948648596897633, 10226950177843565242, 8613014907901498945, 17752245299007024501, 14545457151902152860, 2125271017104468229, 12418883704517153381, 11094047861641111043, 4744428939016996621, 15789820616077617985, 14509586557766826489, 1779323502892191392, 17968972457398919927, 13802147421923836339, 12582941259955572314, 2020990567415091337, 13978010045231727704, 17689546768358619826, 11528384007317036999, 2618359417083270642, 3753549852118440680, 13228535962792011077, 15403182053862085004, 10649861554998259897, 9000929470176218174, 11389062492256305401, 500620887036826686, 8625567553944072034, 15733192642001552110, 753132133723070045, 5228728489570897049, 389582041216651790, 8588631213633954790, 669211513862054828, 17908659775121116604, 9332847924989009589, 8166434271447653166, 8544849009395095914, 3860525578334871396, 4345601671761104418, 937030866713849413, 2685164310433188421, 2839606542081860254, 11519564051649948432, 4230071297110528347, 1617569726334726089, 7521307815019439815, 4933226523190652149, 17506179749774654459, 3575770659638717104], [12637961544485817273, 3818458527534226668, 5989907761206428656, 1867213369914493448, 18202999108773250670, 14920207781042527026, 11365990254256799329, 2472543827968431211, 16256329921569738162, 9388577684719729986, 1255687518490742792, 14611053582827513653, 6541853686353372120, 4291148726508063891, 4222104950641859721, 5639304321561432961, 16596415678156588147, 12784801426352593392, 15443681247396818976, 2069923223560016508, 8159724669666931091, 237642784737007024, 5430898001440415171, 12478188433502465892, 2683851849242339432, 13145746382847899685, 5593476432530716789, 7661341034455639862, 16724915320026526753, 10991773192987913993, 17686696736067726827, 6553995026394159144, 8662409087638866116, 11686874914946865932, 5408273939014519806, 6158113068125158924, 228261331761882995, 4895201551327605491, 10617935710765542546, 9906718555789213290, 8892669301161103683, 12864705010269108342, 6013118169795933611, 8897888853250622559, 2547983454923584235, 5970672836739949974, 8260231415748216880, 12183812260815304087, 8746032047132054324, 9076677267814865422, 8385794308690203861, 17407984318001305001, 11708918714706876085, 12276158571980551820, 2609341930051545016, 15390670918884507950, 5770417693926763592, 1659956574646495230, 14895514114867823092, 4874711997426506854, 10406657296378546202, 17580407650024028039, 720849228998899676, 3925069935132546553], [7791478645377894060, 7545840553330059374, 233598244603919348, 8946537674360828191, 13458351465031681544, 10511316262631658923, 12120693143904860693, 5124231698859408407, 17885917503070942880, 12554815547305127874, 11066793962007676870, 12380233830999578802, 2031508547648947730, 11931623986141180889, 17920017759161566775, 6704476272217874431, 4703955248718923016, 17066538912054270242, 5909186698440233690, 18301382549390844583, 11901942376164316942, 13972876052868914996, 8277386835080684836, 4798564496290197357, 1747286991640684798, 2700746000121608670, 9359478599183426793, 10790073450328365151, 11476815053850371644, 14599381143212907969, 6178966195755250899, 8506417853359736501, 4146883103854988849, 14516968795726946111, 1816058456216431164, 8977902750258702526, 257472656808695542, 8851962166128852715, 2198288591313193145, 10151812699590078245, 9565197583497369880, 18378366699965293191, 5617433452025544320, 11787275131789341806, 6779244591953214586, 4904207676837786845, 3338132131496535468, 3116076104692186327, 13222668365666399627, 10617668376851065773, 9001359680079018719, 9230564775716765855, 1525355680602348022, 12410193164519343985, 11202191315909411466, 7526760672916387442, 14688711286787036279, 1400827163978364548, 16595863801460271558, 6662985756399142363, 6069200313186252076, 2429149444365335144, 10610254696019801256, 13970297598436643667], [13652629899706021678, 4731777230307226708, 14378828634840791597, 1877859774239937583, 4237538877862559254, 1018524078518231376, 9684374190085442386, 1622019555674800803, 13268625408711167535, 829662127955872157, 5340639568839557967, 9761966545978775655, 17880386264116908092, 12769654295370133347, 8837704440003632814, 16847672901975207568, 7978322579254113617, 6819294132587663727, 3313624676119614112, 13356447365546244367, 15758088212005168100, 220008261979604882, 364197342568468945, 4205095054429332713, 5321003861315607285, 7236226988542099354, 2374665111800249111, 11823321657678975059, 14521869248077396776, 6947731965282290857, 10283375502670121943, 6303972817748744707, 16231387414934486933, 11647907543538597982, 2775852892491190135, 17433207219432152689, 11217653845702203393, 17852635440439380794, 12258336801145141522, 7343484145168966773, 6236731188487098123, 5425897016790557464, 11343463962833237345, 6330097802754107018, 15766931889764733078, 7366186306508713172, 1477659074416983571, 4490054239204581592, 7887834010679347853, 11007055485965513939, 3733406945676192806, 9105883417145013582, 6440190596926720450, 1742850934887353199, 11152779982359076530, 17211079224535815291, 15933606946580699825, 4615737521908403625, 9302680250986063579, 7616158555479482396, 9418315437004907429, 1595672542964531770, 1612167222540211644, 12655355949777770272], [410565299112684976, 1722498604142210871, 6890590914018715154, 13640003750310119720, 9940763909770684404, 1200470911022265902, 3553655146888801633, 9871944275383190226, 17904098206209604839, 2858599770734344540, 4523323877256132664, 6616511942133921902, 3010263998596676355, 12494862926904144391, 11015097600965212997, 12996531469813755251, 13749457160604627607, 13871827184725801025, 9166692862118181880, 6889773508473211476, 16881480063004019223, 5648909416009396665, 13942266579726965027, 14034653408757937766, 12138508857070083737, 14966119059556146090, 12608623839457348521, 1576586476503797955, 16800809999590079229, 13385749112768247839, 14648150202512414223, 14100364429510569522, 9900432302233855234, 6863529508818250538, 14270192353796582529, 16059980605699770193, 219597227328877597, 1500818542829593081, 11477286435170972055, 3454932545976109448, 18248832777612451318, 8562720967711475093, 15806435802430525034, 8656289291466571466, 3904341273987863267, 710093797062713916, 5190081202095437742, 5905867208841848672, 14245759917035306401, 16925698706868507994, 3375186042322494175, 1733500171427950969, 15509594745206822524, 13454896373886673573, 16844475268356297768, 1721965743325330751, 7223385307691443057, 7277733711323650134, 5977095451481553028, 1588924474315005812, 707322705874410150, 10180003832225618874, 6763633589017119638, 7611315943220886111], [7779039402743132517, 13851779896966321569, 15987857671235379365, 14606277739456590605, 13447505797397739296, 12659530164295032529, 17998824432140881674, 4772522714979105953, 4653587645322139524, 15040393048614923961, 316632766690077840, 15063731898870247396, 12683376704005591459, 2714132877011286197, 3443975354105110750, 12232629352636889353, 15397183657892026034, 2010664901444003187, 3853825449081474265, 14850144751384145061, 9112493420528819431, 9791908641507472073, 8217232513757085534, 12335645964728132474, 219259782991661223, 17347368170908857433, 6201923802765293595, 12659750292130820022, 17612726657514423861, 8828813493368761425, 6297840668427503489, 17905372135275708943, 5720580984640384755, 2824353444361209553, 15712119034391748415, 5952231055066253854, 5600331442317590795, 6557570197693625931, 6249413803402434895, 9690748460177980353, 13013725978092731570, 6735139379886779728, 1550082417580260311, 3036755452292449464, 17346751523485322105, 1962430260833396262, 2042574880080333986, 13280624353094206692, 1685621892189337604, 2371381896420790070, 3537590533034258263, 5952700750662199822, 729319381398739142, 15384826633650075026, 9954229182363956943, 3352163997206221715, 15640494383146857415, 10472251784054118663, 7810386956658665861, 17951145366536806678, 5174482578836589669, 5413836548169391540, 13879359491023419067, 8782098683948979974], [18245404259401097121, 13186754016615412249, 642127417358658197, 1296177045564272079, 17589397223823950865, 14523623011465281463, 11211130958699062056, 3492780116171781776, 13786653282786517211, 7641045024080616275, 12475292851740543948, 1116141457246000335, 12662674839718353648, 9601170860421582421, 9310079184193082814, 17414719440917112560, 14952553910592039262, 9115741324718783365, 13746619394238059645, 10044590025030364360, 16874212557702271773, 493188323901247095, 4215814581836693122, 16666044150336254646, 2526564779204022029, 14163287002718528347, 13877409874015848828, 10222262147225117273, 6715852608850933262, 2984090932869074626, 16585123996473141130, 3096511859761879243, 7167520552644588175, 6761200453406928940, 46964123436352047, 1221684172796669754, 2749408524197725382, 3793539706748311352, 9890232931133321326, 18235542761269834950, 4568914370141352109, 9046163437516221463, 3459802273071207172, 6713589307631160206, 18072288418347620001, 6316856164593589835, 6749526926788408958, 1990240634583131481, 7625532927541594196, 3531772364295245696, 18131220125692617840, 8054763731935814830, 3896218752581659194, 14297627923994727894, 12031396626464218031, 8249314744879303378, 10843113471157340059, 5804233977104792903, 10689907817679286848, 10084695896890845582, 16365435189465024988, 17981593376875203703, 16089131860759705890, 5747413956356985132], [9451613976857353635, 8015273679363794803, 3590884314337195482, 1846499227436691214, 4353548331289436959, 1248896864933519698, 16388059604114006463, 10584645945018107311, 7193374820157234059, 11214996466070321449, 264709480090191434, 3185178089184654172, 7930254917635729015, 1877637295783422482, 2670914269592086783, 7366982105200657119, 6339512674970094925, 6919238607071847760, 671403014910945163, 12288650714751361383, 12207270478753210350, 6737174068488988699, 16333141575696767082, 13594119917075858257, 18241380355336716696, 2341903752991739265, 2422459995216280307, 17821773082315287047, 10921649222379293695, 7201371350694195659, 12201584129715538614, 16381830107336552877, 16770877910206858631, 16776520204788495017, 2496383087971068879, 6594422255270758732, 5435106299248995913, 4881401643836032687, 17242618807786460058, 11908533745967178146, 11715397755033489826, 13129097279879464867, 14493613979519877242, 16403606009090987573, 5750791282612031110, 293649581219247500, 3740241154940746322, 12588012928482729464, 6313039033895981898, 6404861155434595194, 14885310723562268727, 11283419688200035612, 15748437757606776239, 3929043537785664547, 4923055369510827126, 7311656456631241352, 10554579659059309716, 6915128464696150691, 11557244127925733958, 16238112453302015243, 15744307161948952304, 5679788092094357298, 16382508299487795787, 4713891086822135475], [17166478691628827635, 14520007240373134788, 1471413662452203934, 2297429591686295660, 16839413378879526416, 2291806829714315855, 689317263949285550, 966075922398449681, 17484996431671855613, 9059062957415367465, 866456075568028810, 3192885795317939488, 1092877965338079272, 2564153327648940986, 9091966783146929173, 12106194077252494484, 4202918962756905654, 6889795259773189663, 16972821685023861180, 7770843591805869795, 13704521432888007950, 2751534423821416570, 12207601849690138166, 17700947041525779586, 15963554181270237717, 18234539710548073555, 3065210608414747544, 7680359136095589617, 7807305802873916025, 16125146695453771701, 2153980829613740870, 7431820434170293496, 17494150178483743263, 17104148102621597964, 9570129204928501793, 13572640501919940623, 6949215867211628152, 2921307502083670877, 13410763082995895931, 9059069252511897786, 3016100502838956743, 11221438358597351405, 14137775482581840339, 3307489079396475959, 5064438703479623445, 6118755975178279406, 14134258095348846202, 12035368951191335746, 15713350913004668283, 1918726683636310289, 5219032051546899174, 8041789940760549571, 16379903910729668375, 3219986090994201704, 18221531883209605641, 452303657869888521, 17485857767320459273, 9177286152463971644, 4929430374193380628, 9767662306807934123, 445040373608105346, 16563641077680291286, 16596223245084918681, 8749443017755779264], [5563780445145180537, 5359426854540974324, 7945315652555689579, 1492293258872562237, 464860839337335219, 12106461765068657127, 14521807264272938728, 3595314330920008335, 13958761533281559592, 13990094445575080934, 16630069799544020024, 1142959613162850682, 5544270695360673468, 13059376759593373760, 304643084502334719, 7142072976416882680, 5497962135526056041, 16917184095954237778, 6843297025242932122, 14577176609448409772, 16543063273026810764, 5665290636354182332, 7368244154757485055, 12630636078381024810, 8334828477248557690, 5244078326641330582, 8429085013381930792, 7511956299216381822, 13577964029668418145, 16557825241409518776, 17010379894699112065, 11932160557949447774, 4240791851323477387, 15796058082803459839, 6742422863556561924, 10954433256008392123, 4465252140150682739, 17820666727728288506, 14516246420908467170, 9716783996565228612, 17715353586115937349, 9560355650885983091, 17881075047615718940, 2465071479388192148, 12537371603827554606, 10280161128436187186, 12146483500456338867, 2795742228793975255, 13830518861442499841, 4352356427666751707, 6954751247267169544, 7989200661841776072, 7148005892953261932, 12562477136038449782, 5469629271570070846, 10453713763345703896, 17041306393407415502, 4974082237032810276, 7656554183802656761, 3291143122114181009, 15244787740741796578, 8785032277347600813, 14968766957161301709, 6785894649422732261], [12224584277147654900, 10793857552019097996, 2693288903257276763, 9994627907114311923, 9172959317274889458, 1663211770427348413, 17660697981440280537, 11336169709243575657, 7752514774232431883, 5520237446574497386, 5846072229811501151, 11072767691286735805, 8352466952705881193, 2433318303638969513, 16537649144375763456, 18057983762898467358, 3258936896081743139, 11640401666768608511, 8815668044066938805, 1437501025367235862, 2626821906108675295, 4280142304493557284, 14865151638793471287, 6882544772144627105, 8142867152852588204, 16222248618547519325, 11926754631202639644, 4687480335195960103, 18294321831493781704, 17698735594572835088, 15452243012232071383, 7972544578700248301, 13569394092655944900, 13384439400079690419, 2783307594004910149, 1624220055914749375, 9652280200827010566, 1255264200016732244, 1107976973739451294, 17490509062956924787, 5015615228809179402, 5563567848163154755, 16720379463224293845, 13906071206186618009, 8039740992078292929, 6128540825494942967, 8617950979129772623, 15303753289288015724, 9201162739356610771, 18249808858647996816, 700620284131258128, 1279012369309833036, 9286041732253884136, 10601684155029915001, 5476844454108727530, 13127920948885198868, 18170122247108373025, 6351534181007554530, 6248649940443946634, 6381473368112409579, 7079291824486616497, 10840169308708618388, 9400373599522668107, 16913855778665014168], [465896592257081699, 3418921441922833610, 15650165664031377979, 3079312938400696612, 16658240818497351578, 7960274597204266373, 18315751782997811879, 8369751161774183039, 15833822692026204550, 3716616784199283336, 18367932165077230107, 8738976018273339195, 207217259744311014, 15032138863978486361, 5262284209621630799, 16852601898460823060, 2542473800045220257, 9138310482711095884, 448044005912378138, 16390691507908316936, 7993565736066263326, 4843827855966551781, 14238407711293712978, 5827907400757867706, 13800047427489963228, 1513142246300785974, 3197024874167285791, 2254481322099652857, 12773779635317770, 17017738983285227095, 8238797339142002482, 17048319017918142469, 12861885820083422349, 14630541415015258044, 5935932670656915538, 4970054299516488768, 13130138400473960495, 6595104082530171960, 10971295409079737399, 5437877816153830119, 611772500480429092, 1669036150349419150, 4793494470624569729, 4477907245564894888, 16480066245016189301, 5644898329953081122, 11860750609533110820, 17945745494490133225, 366575838882621494, 18215416410904842881, 14962842769880148905, 9918741804801197758, 8027848215917524104, 10430481770702895558, 10749170983465616564, 5671969822527221760, 10983049064769479332, 1886097196068570761, 16477736718805365119, 7783711824761709925, 12201314913917661611, 14806551723298467418, 5129823777915577030, 9671334794555851570]], side_to_move: 1824412900670340853, en_passant_square_file: [7868963312023991339, 10964446667095117086, 17804051506177768464, 4472259559484458305, 18075143448622176165, 7490649667062053202, 3905869836776920574, 1920488474040085804], castling_rights: [1657968874664877037, 5639102808121395885, 7272733256024614265, 11864137835328840642, 11151353401199634595, 12637484436744206508, 11318084570055320765, 13542206266774110883, 15328763119396809796, 533864607033547165, 5068213126805963243, 1148476244114726247, 2346037464748702132, 5043345032106541740, 1989979229529958305, 4470902676859291578] };

/// An almost unique index number for a chess position.
#[derive(PartialEq, Debug, Clone, Copy, Eq)]
pub struct Zobrist(u64);

impl Zobrist {
    /// Empty hash
    pub const EMPTY: Self = Self(0);

    /// Toggles en passant square from hash.
    pub const fn xor_en_passant(&mut self, en_passant_square: &Square) {
        self.0 ^= ZOBRIST_RANDOMS.en_passant_square_file[en_passant_square.file() as usize];
    }

    /// Toggles castling rights from hash.
    pub const fn xor_castling_rights(&mut self, castling_rights: &CastlingRights) {
        self.0 ^= ZOBRIST_RANDOMS.castling_rights[castling_rights.internal_value() as usize];
    }

    /// Toggles piece from hash.
    pub const fn xor_piece(&mut self, piece_index: usize, square_index: usize) {
        self.0 ^= ZOBRIST_RANDOMS.piece_arrays[piece_index][square_index];
    }

    /// Toggles side to move from hash.
    pub const fn flip_side_to_move(&mut self) {
        self.0 ^= ZOBRIST_RANDOMS.side_to_move;
    }

    /// Take lower 32 bits from hash.
    #[must_use]
    pub const fn lower_u32(&self) -> u32 {
        self.0 as u32
    }

    /// Uniformly map zobrist key into an integer no larger than `size`.
    #[must_use]
    pub const fn distribute(&self, size: usize) -> u64 {
        ((self.0 as u128 * size as u128) >> 64) as u64
    }

    #[must_use]
    pub const fn modulo(&self, size: u64) -> u64 {
        self.0 % size
    }

    /// Computes the position zobrist key.
    #[must_use]
    pub fn compute(board: &Board) -> Self {
        let mut key = Self::EMPTY;

        for (piece, bit_board) in board.bit_boards.iter().enumerate() {
            let mut bit_board = *bit_board;
            consume_bit_board!(bit_board, square {
                key.xor_piece(piece, square.usize());
            });
        }

        if !board.white_to_move {
            key.flip_side_to_move();
        }

        if let Some(en_passant_square) = board.game_state.en_passant_square {
            key.xor_en_passant(&en_passant_square);
        }

        key.xor_castling_rights(&board.game_state.castling_rights);

        key
    }

    /// Computes the pawn zobrist key.
    #[must_use]
    pub fn pawn_key(board: &Board) -> Self {
        let mut key = Self::EMPTY;

        let mut black_pawns = *board.get_bit_board(Piece::BlackPawn);
        consume_bit_board!(black_pawns, square {
            key.xor_piece(Piece::BlackPawn as usize, square.usize());
        });
        let mut white_pawns = *board.get_bit_board(Piece::WhitePawn);
        consume_bit_board!(white_pawns, square {
            key.xor_piece(Piece::WhitePawn as usize, square.usize());
        });

        key
    }

    /// Computes the minor piece zobrist key.
    #[must_use]
    pub fn minor_piece_key(board: &Board) -> Self {
        let mut key = Self::EMPTY;

        for piece in [
            Piece::BlackKnight,
            Piece::WhiteKnight,
            Piece::BlackBishop,
            Piece::WhiteBishop,
            Piece::BlackKing,
            Piece::WhiteKing,
        ] {
            let mut pieces = *board.get_bit_board(piece);
            consume_bit_board!(pieces, square {
                key.xor_piece(piece as usize, square.usize());
            });
        }

        key
    }
}
