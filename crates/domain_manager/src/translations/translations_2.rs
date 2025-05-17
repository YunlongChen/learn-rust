#![allow(clippy::match_same_arms, clippy::match_wildcard_for_single_variants)]

use crate::translations::types::language::Language;

pub fn new_version_available_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "A newer version is available!",
        Language::IT => "Una versione più recente è disponibile!",
        Language::RU => "Новая версия доступна!",
        Language::EL => "Μια νεότερη έκδοση είναι διαθέσιμη!",
        // Language::FA => "یک نسخه جدیدتر روی GitHub موجود است",
        Language::SV => "En nyare version finns tillgänglig!",
        Language::FI => "Uudempi versio saatavilla!",
        Language::DE => "Eine neue Version ist verfügbar!",
        Language::TR => "Daha yeni bir versiyon mevcut!",
        Language::ES => "Hay una nueva versión disponible!",
        Language::KO => "새로운 버전이 출시되었습니다!",
        Language::ZH => "新版本已在 Github 发布!",
        Language::ZH_TW => "有可用的新版本！",
        Language::UK => "Нова версія доступна!",
        Language::RO => "O versiune nouă este disponibilă!",
        Language::PL => "Nowsza wersja jest dostępna!",
        Language::FR => "Une nouvelle version est disponible!",
        Language::JA => "新しいバージョンが利用可能になりました!",
        Language::UZ => "Yangi versiya mavjud!",
        Language::PT => "Uma nova versão está disponível!",
        Language::VI => "Phiên bản mới đã sẵn sàng!",
        Language::ID => "Versi baru tersedia!",
    }
}

pub fn inspect_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Inspect",
        Language::IT => "Ispeziona",
        Language::FR => "Inspecter",
        Language::ES => "Inspeccionar",
        Language::PL => "Sprawdź",
        Language::DE => "Inspizieren",
        Language::RU => "Инспектировать",
        Language::SV => "Inspektera",
        Language::FI => "Tarkastele",
        Language::TR => "İncele",
        // Language::FA => "بازرسی",
        Language::KO => "검사",
        Language::ZH => "检查",
        Language::ZH_TW => "檢查",
        Language::UK => "Перевірити",
        Language::RO => "Inspectați",
        Language::JA => "検査",
        Language::UZ => "Tekshirish",
        Language::PT => "Inspecionar",
        Language::VI => "Quan sát",
        Language::ID => "Memeriksa",
        _ => "Inspect",
    }
}

pub fn connection_details_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Connection details",
        Language::IT => "Dettagli della connessione",
        Language::RU => "Подробнее о соединении",
        Language::SV => "Anslutningsdetaljer",
        Language::FI => "Yhteyden tiedot",
        Language::DE => "Verbindungsdetails",
        Language::TR => "Bağlantı detayları",
        // Language::FA => "مشخصات اتصال",
        Language::ES => "Detalles de la conexión",
        Language::KO => "연결 상세",
        Language::ZH => "连接详情",
        Language::ZH_TW => "連線詳細資訊",
        Language::UK => "Деталі зʼєднання",
        Language::RO => "Detalii conexiune",
        Language::PL => "Szczegóły połączenia",
        Language::FR => "Détails de la connexion",
        Language::JA => "接続の詳細",
        Language::UZ => "Ulanish tafsilotlari",
        Language::PT => "Detalhes da conexão",
        Language::VI => "Thông tin kết nối",
        Language::ID => "Rincian koneksi",
        _ => "Connection details",
    }
}

// refers to bytes or packets dropped because they weren't processed fast enough
pub fn dropped_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Dropped",
        Language::IT => "Persi",
        Language::RU => "Потеряно",
        Language::SV => "Tappade",
        Language::FI => "Pudotetut",
        Language::DE => "Verlorene",
        Language::TR => "Düşen",
        // Language::FA => "رها شده",
        Language::ES | Language::PT => "Perdidos",
        Language::KO => "손실",
        Language::ZH => "丢计",
        Language::ZH_TW => "丟棄",
        Language::UK => "Пропущені",
        Language::RO => "Pierdute",
        Language::PL => "Utracone",
        Language::FR => "Perdus",
        Language::JA => "ドロップした",
        Language::UZ => "Yig'ilgan",
        Language::VI => "Mất",
        Language::ID => "Dihapus",
        _ => "Dropped",
    }
}

pub fn data_representation_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Data representation",
        Language::IT => "Rappresentazione dei dati",
        Language::RU => "Показывать в виде", // there is selector below: "байтов" or "пакетов"
        Language::SV => "Datarepresentation",
        Language::FI => "Tietojen esitys",
        Language::DE => "Daten Darstellung",
        Language::TR => "Veri gösterimi",
        // Language::FA => "بازنمایی داده ها", // TODO: or نمایندگی داده ها depending on context
        Language::ES => "Representación de los datos",
        Language::KO => "데이터 단위",
        Language::ZH => "图表数据",
        Language::ZH_TW => "資料呈現方式",
        Language::UK => "Відображення даних",
        Language::RO => "Reprezentarea datelor",
        Language::PL => "Reprezentacja danych",
        Language::FR => "Représentation de données",
        Language::JA => "データ表示",
        Language::UZ => "Ma'lumotlarni taqdim etish",
        Language::PT => "Representação dos dados",
        Language::VI => "Miêu tả dữ liệu",
        Language::ID => "Penyajian ulang data",
        _ => "Data representation",
    }
}

pub fn host_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Network host",
        Language::IT => "Host di rete",
        Language::RU => "Сетевой хост",
        Language::SV => "Nätverksvärd",
        Language::FI => "Verkkoisäntä",
        Language::DE => "Netzwerk-Host",
        Language::TR => "Ağ sunucusu",
        // Language::FA => "میزبان شبکه",
        Language::ES => "Host de red",
        Language::KO => "네트워크 호스트",
        Language::ZH => "主机",
        Language::ZH_TW => "網路主機",
        Language::UK => "Мережевий хост",
        Language::RO => "Host rețea",
        Language::PL => "Host sieciowy",
        Language::FR => "Host réseaux",
        Language::JA => "ネットワーク ホスト",
        Language::UZ => "Tarmoq serveri",
        Language::PT => "Host da rede",
        Language::VI => "Máy chủ",
        Language::ID => "Jaringan asal",
        _ => "Network host",
    }
}

pub fn only_top_30_items_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Only the top 30 items are displayed here",
        Language::IT => "Solo i 30 maggiori elementi sono mostrati qui",
        Language::RU => "Показываются только первые 30 элементов",
        Language::SV => "Endast de 30 främsta föremål visas här",
        Language::FI => "Vain 30 parasta kohteita näytetään tässä",
        Language::DE => "Nur die obersten 30 Elemente werden hier angezeigt",
        Language::TR => "Sadece ilk 30 öğeler burda gösterilmektedir",
        // Language::FA => "تنها ۳۰ موارد برتر در اینجا نمایش داده شده اند",
        Language::ES => "Aquí sólo se muestran los 30 primeros elementos",
        Language::KO => "상위 30개의 아이템만 노출됩니다",
        Language::ZH => "仅展示前 30 个项目",
        Language::ZH_TW => "此處僅顯示前 30 個項目",
        Language::UK => "Тут відображаються лише перші 30 елементів",
        Language::RO => "Doar primele 30 de articole sunt afișate aici",
        Language::PL => "Tylko 30 pierwszych rzeczy jest wyświetlanych",
        Language::FR => "Seuls les 30 premiers articles sont affichés ici",
        Language::JA => "上位 30 件のアイテムのみが表示されます",
        Language::UZ => "Bu erda faqat dastlabki 30 ta buyumlar ko'rsatiladi",
        Language::PT => "Apenas os 30 melhores unid são expostos aqui",
        Language::VI => "Chỉ có 30 mục gần nhất được hiển thị ở đây",
        Language::ID => "Hanya 30 teratas yang ditampilkan disini",
        _ => "Only the top 30 items are displayed here",
    }
}

// pub fn sort_by_translation(language: Language) -> &'static str {
//     match language {
//         Language::EN => "Sort by",
//         Language::IT => "Ordina per",
//         Language::RU => "Сортировка",
//         Language::SV => "Sortera efter",
//         Language::FI => "Järjestä",
//         Language::DE => "Sortieren nach",
//         Language::TR => "Şuna göre sırala",
//         // Language::FA => "مرتب سازی بر اساس",
//         Language::ES | Language::PT => "Ordenar por",
//         Language::KO => "정렬",
//         Language::ZH => "排序",
//         Language::ZH_TW => "排序依據",
//         Language::UK => "Сортувати за",
//         Language::RO => "Filtrează după",
//         Language::PL => "Sortuj według",
//         Language::FR => "Trier par",
//         Language::JA => "ソート",
//         Language::UZ => "Saralash turi",
//         Language::ID => "Urut berdasarkan",
//         _ => "Sort by",
//     }
// }

pub fn local_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Local network",
        Language::IT => "Rete locale",
        Language::RU => "Локальная сеть",
        Language::SV => "Lokalt nätverk",
        Language::FI => "Paikallinen verkko",
        Language::DE => "Lokales Netzwerk",
        Language::TR => "Yerel ağ",
        // Language::FA => "شبکه محلی",
        Language::ES => "Red local",
        Language::KO => "로컬 네트워크",
        Language::ZH => "局域网",
        Language::ZH_TW => "區域網路",
        Language::UK => "Локальна мережа",
        Language::RO => "Rețea locală",
        Language::PL => "Sieć lokalna",
        Language::FR => "Réseau local",
        Language::JA => "ローカル ネットワーク",
        Language::UZ => "Mahalliy tarmoq",
        Language::PT => "Rede local",
        Language::VI => "Mạng nội bộ",
        Language::ID => "Jaringan lokal",
        _ => "Local network",
    }
}

pub fn unknown_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Unknown location",
        Language::IT => "Localizzazione sconosciuta",
        Language::RU => "Неизвестный регион",
        Language::SV => "Okänd plats",
        Language::FI => "Tuntematon sijanti",
        Language::DE => "Unbekannter Ort",
        Language::TR => "Bilinmeyen yer",
        // Language::FA => "محل نامعلوم",
        Language::ES => "Ubicación desconocida",
        Language::KO => "알 수 없는 위치",
        Language::ZH => "未知",
        Language::ZH_TW => "未知位置",
        Language::UK => "Невідоме місцезнаходження",
        Language::RO => "Locație necunoscută",
        Language::PL => "Nieznana lokalizacja",
        Language::FR => "Localisation inconnue",
        Language::JA => "不明なロケーション",
        Language::UZ => "Noma'lum joylashuv",
        Language::PT => "Localização desconhecida",
        Language::VI => "Không rõ địa điểm",
        Language::ID => "Lokasi tidak diketahui",
        _ => "Unknown location",
    }
}

pub fn your_network_adapter_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Your network adapter",
        Language::IT => "La tua scheda di rete",
        Language::RU => "Ваш сетевой адаптер",
        Language::SV => "Din nätverksadapter",
        Language::FI => "Sinun verkkosovitin",
        Language::DE => "Dein Netzwerk-Adapter",
        Language::TR => "Ağ adaptörün",
        // Language::FA => "مبدل شبکه شما",
        Language::ES => "Su adaptador de red",
        Language::KO => "네트워크 어댑터",
        Language::ZH => "你的网络适配器",
        Language::ZH_TW => "您的網路介面卡",
        Language::UK => "Ваш мережевий адаптер",
        Language::RO => "Adaptorul dvs. de rețea",
        Language::PL => "Twój adapter sieciowy",
        Language::FR => "Votre carte réseau",
        Language::JA => "自身のネットワーク アダプター",
        Language::UZ => "Sizning tarmoq adapteringiz",
        Language::PT => "Seu adaptador de rede",
        Language::VI => "Network adapter của bạn",
        Language::ID => "Adaptor jaringan kamu",
        _ => "Your network adapter",
    }
}

pub fn socket_address_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Socket address",
        Language::IT => "Indirizzo del socket",
        Language::RU => "Адрес сокета",
        Language::SV => "Socketadress",
        Language::FI => "Socket osoite",
        Language::DE => "Socket Adresse",
        Language::TR => "Soket adresi",
        // Language::FA => "پریز شبکه",
        Language::ES => "Dirección del socket",
        Language::KO => "소켓 어드레스",
        Language::ZH => "套接字地址",
        Language::ZH_TW => "Socket 位址",
        Language::UK => "Адреса сокета",
        Language::RO => "Adresa socket-ului",
        Language::PL => "Adres gniazda",
        Language::FR => "Adresse du socket",
        Language::JA => "ソケット アドレス",
        Language::UZ => "Soket manzili",
        Language::PT => "Endereço da socket",
        Language::VI => "Địa chỉ socket",
        Language::ID => "Alamat sambungan",
        _ => "Socket address",
    }
}

pub fn mac_address_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "MAC address",
        Language::IT => "Indirizzo MAC",
        Language::RU => "MAC адрес",
        Language::SV => "MAC-adress",
        Language::FI => "MAC-osoite",
        Language::DE => "MAC Adresse",
        Language::TR => "MAC adresi",
        // Language::FA => "آدرس MAC",
        Language::ES => "Dirección MAC",
        Language::KO => "맥 어드레스",
        Language::ZH => "MAC 地址",
        Language::ZH_TW => "MAC 位址",
        Language::UK => "MAC-адреса",
        Language::RO => "Adresa MAC",
        Language::PL => "Adres MAC",
        Language::FR => "Adresse MAC",
        Language::JA => "MAC アドレス",
        Language::UZ => "MAC manzili",
        Language::PT => "Endereço MAC",
        Language::VI => "Địa chỉ MAC",
        Language::ID => "Alamat MAC",
        _ => "MAC address",
    }
}

pub fn source_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Source",
        Language::IT => "Sorgente",
        Language::RU => "Источник",
        Language::SV => "Källa",
        Language::FI => "Lähde",
        Language::DE => "Quelle",
        Language::TR => "Kaynak",
        // Language::FA => "منبع",
        Language::ES => "Origen",
        Language::KO => "소스",
        Language::ZH => "源",
        Language::ZH_TW => "來源",
        Language::UK => "Джерело",
        Language::RO => "Sursă",
        Language::PL => "Źródło",
        Language::FR => "Source",
        Language::JA => "送信元",
        Language::UZ => "Manba",
        Language::PT => "Fonte",
        Language::VI => "Nguồn",
        Language::ID => "Asal",
        _ => "Source",
    }
}

pub fn destination_translation(language: Language) -> &'static str {
    match language {
        Language::EN | Language::SV => "Destination",
        Language::IT => "Destinazione",
        Language::RU => "Получатель",
        Language::FI => "Määränpää",
        Language::DE => "Ziel",
        Language::TR => "Hedef",
        // Language::FA => "مقصد",
        Language::ES | Language::PT => "Destino",
        Language::KO => "목적지",
        Language::ZH => "目标",
        Language::ZH_TW => "目的地",
        Language::UK => "Призначення",
        Language::RO => "Destinație",
        Language::PL => "Miejsce docelowe",
        Language::FR => "Destination",
        Language::JA => "送信先",
        Language::UZ => "Qabul qiluvchi",
        Language::VI => "Đích",
        Language::ID => "Tujuan",
        _ => "Destination",
    }
}

pub fn fqdn_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Fully qualified domain name",
        Language::IT => "Nome di dominio completo",
        Language::RU => "Полное доменное имя",
        Language::SV => "Fullständigt domännamn",
        Language::FI => "Täysin määritelty verkkotunnus",
        Language::DE => "Vollständig qualifizierter Domain Name",
        Language::TR => "Tam nitelikli alan adı",
        // Language::FA => "نام دامنه جامع الشرایط",
        Language::ES => "Nombre de dominio completo",
        Language::KO => "절대 도메인 네임",
        Language::ZH | Language::JA => "FQDN",
        Language::ZH_TW => "FQDN",
        Language::UK => "Повністю визначене доменне ім'я",
        Language::RO => "Nume de domeniu complet calificat",
        Language::PL => "Pełna nazwa domeny",
        Language::FR => "Nom de domaine complètement qualifié",
        Language::UZ => "To'liq domen nomi",
        Language::PT => "Nome de domínio completo",
        Language::VI => "Tên miền đầy đủ",
        Language::ID => "Nama domain yang memenuhi syarat",
        _ => "Fully qualified domain name",
    }
}

pub fn administrative_entity_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Autonomous System name",
        Language::IT => "Nome del sistema autonomo",
        Language::RU => "Имя автономной системы",
        Language::SV => "Administrativ enhet",
        Language::FI => "Autonomisen järjestelmän nimi",
        Language::DE => "Name des autonomen Systems",
        Language::TR => "Yönetim varlığı",
        // Language::FA => "واحد اجرایی", // TODO: or واحد اداری depending on context
        Language::ES => "Nombre del sistema autónomo",
        Language::KO => "관리 엔티티",
        Language::ZH => "ASN 信息",
        Language::ZH_TW => "ASN 資訊",
        Language::UK => "Адміністративна одиниця",
        Language::RO => "Numele sistemului autonom",
        Language::PL => "Nazwa autonomicznego systemu",
        Language::FR => "Nom du système autonome",
        Language::JA => "AS 名",
        Language::UZ => "Avtonom tizim nomi",
        Language::PT => "Entidade administrativa",
        Language::VI => "Tên Autonomous System",
        Language::ID => "Nama System Otomatis",
        _ => "Autonomous System name",
    }
}

pub fn transmitted_data_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Transmitted data",
        Language::IT => "Dati trasmessi",
        Language::RU => "Передано данных",
        Language::SV => "Överförd data",
        Language::FI => "Lähetetty data",
        Language::DE => "Übermittelte Daten",
        Language::TR => "Aktarılan veri",
        // Language::FA => "دادهٔ منتقل شده",
        Language::ES => "Datos transmitidos",
        Language::KO => "수신된 데이터",
        Language::ZH => "数据传输",
        Language::ZH_TW => "已傳輸的資料",
        Language::UK => "Передані дані",
        Language::RO => "Date transmise",
        Language::PL => "Przesłane dane",
        Language::FR => "Données transmises",
        Language::JA => "転送データ",
        Language::UZ => "Uzatilgan ma'lumotlar",
        Language::PT => "Dados transmitidos",
        Language::VI => "Dữ liệu được truyền",
        Language::ID => "Data terkirim",
        _ => "Transmitted data",
    }
}

pub fn country_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Country",
        Language::IT => "Paese",
        Language::RU => "Страна",
        Language::SV => "Land",
        Language::FI => "Maa",
        Language::DE => "Land",
        Language::TR => "Ülke",
        // Language::FA => "کشور",
        Language::ES | Language::PT => "País",
        Language::KO => "국가",
        Language::ZH => "国家",
        Language::ZH_TW => "國家",
        Language::UK => "Країна",
        Language::RO => "Țară",
        Language::PL => "Kraj",
        Language::FR => "Pays",
        Language::JA => "国",
        Language::UZ => "Davlat",
        Language::VI => "Quốc gia",
        Language::ID => "Negara",
        _ => "Country",
    }
}

pub fn domain_name_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Domain name",
        Language::IT => "Nome di dominio",
        Language::RU => "Доменное имя",
        Language::SV => "Domännamn",
        Language::FI => "Verkkotunnus",
        Language::DE => "Domain Name",
        Language::TR => "Alan adı",
        // Language::FA => "نام دامنه",
        Language::ES => "Nombre de dominio",
        Language::KO => "도메인 네임",
        Language::ZH => "域名",
        Language::ZH_TW => "網域名稱",
        Language::UK => "Доменне ім'я",
        Language::RO => "Nume domeniu",
        Language::PL => "Nazwa domeny",
        Language::FR => "Nom de domaine",
        Language::JA => "ドメイン名",
        Language::UZ => "Domen nomi",
        Language::PT => "Nome do domínio",
        Language::VI => "Tên miền",
        Language::ID => "Nama Domain",
        _ => "Domain name",
    }
}

pub fn only_show_favorites_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Only show favorites",
        Language::IT => "Mostra solo i preferiti",
        Language::RU => "Показывать только избранные",
        Language::SV => "Visa endast favoriter",
        Language::FI => "Näytä vain suosikit",
        Language::DE => "Zeige nur die Favoriten",
        Language::TR => "Sadece favorileri göster",
        // Language::FA => "فقط پسندیده ها را نمایش بده",
        Language::ES => "Mostrar solo los favoritos",
        Language::KO => "즐겨찾기만 보기",
        Language::ZH => "仅显示收藏",
        Language::ZH_TW => "僅顯示我的最愛",
        Language::UK => "Показувати лише улюблені",
        Language::RO => "Arată doar favorite",
        Language::PL => "Pokaż tylko ulubione",
        Language::FR => "Afficher uniquement les favoris",
        Language::JA => "お気に入りのみを表示する",
        Language::UZ => "Faqat sevimlilarni ko'rsatish",
        Language::PT => "Apenas mostrar os favoritos",
        Language::VI => "Chỉ hiển thị mục ưa thích",
        Language::ID => "Hanya tunjukkan favorit",
        _ => "Only show favorites",
    }
}

// pub fn search_filters_translation(language: Language) -> &'static str {
//     match language {
//         Language::EN => "Search filters",
//         Language::IT => "Filtri di ricerca",
//         Language::RU => "Фильтры для поиска",
//         Language::SV => "Sökfilter",
//         Language::FI => "Hakusuodattimet",
//         Language::DE => "Filter suchen",
//         Language::TR => "Arama filtresi",
//         // Language::FA => "صافی های جستجو",
//         Language::ES => "Filtros de búsqueda",
//         Language::KO => "검색 필터",
//         Language::ZH => "搜索条件",
//         Language::ZH_TW => "搜尋篩選器",
//         Language::UK => "Фільтри пошуку",
//         Language::RO => "Filtre de căutare",
//         Language::PL => "Filtry wyszukiwania",
//         Language::FR => "Filtres de recherche",
//         Language::JA => "検索フィルター",
//         Language::UZ => "Qidiruv filtrlari",
//         Language::PT => "Filtros de busca",
//         Language::ID => "Filter Pencarian",
//         _ => "Search filters",
//     }
// }

pub fn no_search_results_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "No result available according to the specified search filters",
        Language::IT => "Nessun risultato disponibile secondo i filtri di ricerca specificati",
        Language::RU => "Ничего не найдено после применения выбранных фильтров",
        Language::SV => "Inga resultat tillgängliga utifrån de angivna sökfilterna",
        Language::FI => "Ei tuloksia saatavilla määritellyille hakusuodattimille",
        Language::DE => "Keine Resultate für die gewählten Filter verfügbar",
        Language::TR => "Belirtilen arama filtrelerine göre herhangi bir sonuç bulunmamaktadır",
        // Language::FA => "هیچ نتیجه ای بر اساس صافی های جستجوی تعیین شده وجود ندارد",
        Language::ES => "Los filtros de búsqueda especificados no generan ningún resultado",
        Language::KO => "해당 검색 필터로 검색된 결과가 없습니다.",
        Language::ZH => "没有符合条件的条目",
        Language::ZH_TW => "根據指定的篩選條件，找不到任何結果",
        Language::UK => "Немає результатів згідно з обраними фільтрами пошуку",
        Language::RO => "Niciun rezultat disponibil conform filtrelor de căutare specificate",
        Language::PL => "Brak wyników zgodnych z określonymi filtrami wyszukiwania",
        Language::FR => "Aucun résultat disponible selon les filtres de recherche spécifiés",
        Language::JA => "指定されたフィルター条件で表示できる結果はありません",
        Language::UZ => "Belgilangan qidiruv filtrlari bo'yicha hech qanday natija mavjud emas",
        Language::PT => "Nenhum resultado disponível de acordo com os filtros selecionados",
        Language::VI => "Không có kết quả nào theo các bộ lọc được chỉ định",
        Language::ID => "Tidak ada hasil berdasarkan filter pencarian spesifik",
        _ => "No result available according to the specified search filters",
    }
}

pub fn showing_results_translation(
    language: Language,
    start: usize,
    end: usize,
    total: usize,
) -> String {
    match language {
        Language::EN => format!("Showing {start}-{end} of {total} total results"),
        Language::IT => format!("Sono mostrati {start}-{end} di {total} risultati totali"),
        Language::RU => format!("Показываются {start}-{end} из {total} общего числа результатов"),
        Language::SV => format!("Visar {start}-{end} av {total} totala resultat"),
        Language::FI => format!("Näytetään {start}-{end} tulosta, kaikista tuloksista {total}"),
        Language::DE => format!("{start}-{end} von insgesamt {total} Resultaten werden angezeigt"),
        Language::TR => format!("{total} sonuç içinde {start}-{end}"),
        // Language::FA => format!("نمایش {start}-{end} از تمامی {total} نتیجه"),
        Language::ES => format!("Mostrando {start}-{end} de {total} resultados totales"),
        Language::KO => format!("총 {total}개의 결과 중 {start}-{end}을(를) 보여줍니다"),
        Language::ZH => format!("显示累计 {total} 条目中第 {start}-{end} 个"),
        Language::ZH_TW => format!("顯示總共 {total} 個結果中的第 {start}-{end} 個"),
        Language::UK => format!("Показано {start}-{end} з {total} загальних результатів"),
        Language::RO => format!("Se afișează {start}-{end} din {total} rezultate"),
        Language::PL => format!("Wyświetlanie {start}-{end} z {total} wyników"),
        Language::FR => format!("Affichage de {start}-{end} de {total} résultats totaux"),
        Language::JA => format!("{total} 件中の {start}-{end} 件を表示"),
        Language::UZ => format!("Jami {total} natijadan {start}-{end} ko'rsatilyapti"),
        Language::PT => format!("Mostrando {start}-{end} de {total} resultados totais"),
        Language::VI => format!("Đang hiển thị {start}-{end} của {total} tổng số kết quả"),
        Language::ID => format!("Menampilkan {start}-{end} dari {total} semua hasil"),
        _ => format!("Showing {start}-{end} of {total} total results"),
    }
}

#[allow(dead_code)]
pub fn color_gradients_translation(language: Language) -> &'static str {
    match language {
        Language::EN => "Apply color gradients",
        Language::IT => "Applica sfumature di colore",
        Language::RU => "Применить цветовой градиент", // recheck
        Language::SV => "Applicera färggradient",
        Language::FI => "Käytä värigradientteja",
        Language::DE => "Farb-Gradienten anwenden",
        Language::TR => "Renk grandyanı uygula",
        // Language::FA => "اعمال گرادیان های رنگ",
        Language::ES => "Aplicar gradientes de color",
        Language::KO => "그라디언트 색상 적용",
        Language::ZH => "应用渐变色",
        Language::ZH_TW => "套用色彩漸層",
        Language::UK => "Застосувати кольорові градієнти",
        Language::RO => "Aplicați gradient de culoare",
        Language::PL => "Zastosuj gradient kolorów",
        Language::FR => "Appliquer des gradients de couleur",
        Language::JA => "グラデーションを適用する",
        Language::UZ => "Rang gradientlarini qo'llang",
        Language::PT => "Aplicar gradientes de cor",
        Language::VI => "Áp dụng color gradients",
        Language::ID => "Aplikasikan gradasi warna",
        _ => "Apply color gradients",
    }
}
