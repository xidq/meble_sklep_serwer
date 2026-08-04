use crate::product::get::get_products_data_by_id;
use crate::zamowienia::{Zamowienie, ZamowieniePozycja};
use sqlx::SqlitePool;

pub struct DaneFirmy {
    nip: String,
    adres: String,
    miasto: String,
    kod_pocztowy: String,
    nazwa: String,
    email: String,
    telefon: String,
}

pub async fn generate_ksef_xml(
    zamowienie: &Zamowienie,
    pozycje: &[ZamowieniePozycja],
    pool: &SqlitePool
) -> anyhow::Result<String> {
    // let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let data_wytworzenia = zamowienie.date.replace(" | ", "T") + "Z";
    let moje_dane = DaneFirmy {
        nip: "76576345765".to_string(),
        adres: "hujemuje 69".to_string(),
        miasto: "Dzikie węże".to_string(),
        kod_pocztowy: "69-666".to_string(),
        nazwa: "Pumpernikiel spzoo".to_string(),
        email: "aa@aa.pa".to_string(),
        telefon: "666666666".to_string(),
    };


    // Budowanie wierszy z pobieraniem danych z bazy
    let mut wiersze_xml = String::new();
    for (i, poz) in pozycje.iter().enumerate() {
        let product = get_products_data_by_id(poz.product_id, pool).await?;

        wiersze_xml.push_str(&format!(
r#"<FaWiersz>
<NrWierszaFa>{}</NrWierszaFa>
    <P_7>{}</P_7>
    <P_8A>szt.</P_8A>
    <P_8B>{}</P_8B>
    <P_9A>{:.2}</P_9A>
    <P_11>{:.2}</P_11>
    <P_12>{:.0}</P_12>
</FaWiersz>"#,
            i + 1, product.name_pl, poz.ilosc, poz.cena, poz.cena * poz.ilosc as f32, poz.vat
        ));
    }

    // kupujący
    let podmiot2_xml = if let Some(fv) = &zamowienie.faktura_dane {
        format!(
r#"<Podmiot2>
    <DaneIdentyfikacyjne>
        <NIP>{}</NIP>
        <Nazwa>{}</Nazwa>
    </DaneIdentyfikacyjne>
    <Adres>
        <KodKraju>PL</KodKraju>
        <AdresL1>{}</AdresL1>
        <AdresL2>{} {}</AdresL2>
    </Adres>
    <DaneKontaktowe>
        <Email>{}</Email>
        <Telefon>{}</Telefon>
    </DaneKontaktowe>
</Podmiot2>"#,
    &fv.nip,
    &fv.nazwa_firmy,
    &fv.ulica.as_deref().unwrap_or(""),
    &fv.kod_pocztowy.as_deref().unwrap_or(""),
    &fv.miasto.as_deref().unwrap_or(""),
    &zamowienie.email.as_deref().unwrap_or(""),
    &zamowienie.tel.as_deref().unwrap_or(""),
        )
    } else {
        format!(
r#"<Podmiot2>
    <DaneIdentyfikacyjne><Nazwa>{} {}</Nazwa></DaneIdentyfikacyjne>
    <Adres><KodKraju>PL</KodKraju><AdresL1>{}</AdresL1><AdresL2>{} {}</AdresL2></Adres>
</Podmiot2>"#,
            zamowienie.imie, zamowienie.nazwisko, zamowienie.lokacja.ulica, zamowienie.lokacja.kod_pocztowy, zamowienie.lokacja.miasto
        )
    };
    let kwota_brutto = zamowienie.cena + zamowienie.vat;
    Ok(format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<Faktura xmlns="http://crd.gov.pl/wzor/2025/06/25/13775/">
    <Naglowek>
        <KodFormularza kodSystemowy="FA (3)" wersjaSchemy="1-0E">FA</KodFormularza>
        <WariantFormularza>3</WariantFormularza>
        <DataWytworzeniaFa>{}</DataWytworzeniaFa>
    </Naglowek>
    <Podmiot1>
        <DaneIdentyfikacyjne>
            <NIP>{}</NIP>
            <Nazwa>{}</Nazwa>
        </DaneIdentyfikacyjne>
        <Adres>
            <KodKraju>PL</KodKraju>
            <AdresL1>{}</AdresL1>
            <AdresL2>{} {}</AdresL2>
        </Adres>
        <DaneKontaktowe>
			<Email>abc@abc.pl</Email>
			<Telefon>667444555</Telefon>
		</DaneKontaktowe>
    </Podmiot1>
    {}
    <Fa>
        <KodWaluty>PLN</KodWaluty>
        <P_1>{}</P_1>
        <P_2>{}</P_2>
        <RodzajFaktury>VAT</RodzajFaktury>
        {}
        <P_13_1>{:.2}</P_13_1>
        <P_15>{:.2}</P_15>
    </Fa>
</Faktura>"#,
        data_wytworzenia, moje_dane.nip, moje_dane.nazwa, moje_dane.adres, moje_dane.kod_pocztowy, moje_dane.miasto,
        podmiot2_xml, zamowienie.date.split('|').next().unwrap_or("").trim(), zamowienie.numer_fv,
        wiersze_xml, zamowienie.cena, kwota_brutto
    ))
}
