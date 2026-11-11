//! Feather Sans, a letra da interface da suíte (assets/fontes, licença OFL 1.1).
//! Gerada pelo `assets/fontes/feather_sans.py`: mudou o desenho, rode o script e troque os `.ttf`.

/// Nome da família, para pedir com `pelicano::Font::with_name`.
pub const FAMILIA: &str = "Feather Sans";

const ARQUIVOS: [&[u8]; 6] = [
    include_bytes!("../assets/fontes/FeatherSans-Regular.ttf"),
    include_bytes!("../assets/fontes/FeatherSans-Italic.ttf"),
    include_bytes!("../assets/fontes/FeatherSans-Bold.ttf"),
    include_bytes!("../assets/fontes/FeatherSans-BoldItalic.ttf"),
    include_bytes!("../assets/fontes/FeatherSans-Thin.ttf"),
    include_bytes!("../assets/fontes/FeatherSans-ThinItalic.ttf"),
];

/// Carrega a Feather Sans e a torna a letra padrão de todas as janelas. Chamar uma vez,
/// antes de abrir a primeira. O que não pedir família (`Font::DEFAULT`) usa ela; o que
/// faltar nela cai nas fontes do sistema; `Font::SYSTEM_SANS` continua a do sistema
/// (o texto das células, que segue o arquivo e não a interface).
pub fn usar_na_interface() {
    for bytes in ARQUIVOS {
        pelicano::load_font(bytes);
    }
    pelicano::set_default_family(FAMILIA);
}

#[cfg(test)]
mod testes {
    #[test]
    fn os_seis_estilos_sao_fontes_truetype_da_familia() {
        // A tabela name guarda o nome em UTF-16 (plataforma Windows).
        let nome: Vec<u8> = super::FAMILIA.encode_utf16().flat_map(u16::to_be_bytes).collect();
        for bytes in super::ARQUIVOS {
            assert_eq!(&bytes[..4], &[0, 1, 0, 0], "cabeçalho TrueType");
            assert!(bytes.windows(nome.len()).any(|j| j == nome), "família na tabela name");
        }
    }
}
