//! Marca vetorial única. A cor da fábrica pertence ao aplicativo;
//! aparência Claro/Escuro/Automático não altera a logo sem opção explícita.
use pelicano::{Icon, Settings, SvgView};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use crate::tema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Produto { FeatherSheet=0, FeatherDocs=1, FeatherSlide=2 }
static PRODUTO: AtomicU8=AtomicU8::new(0);
static COR_PERSONALIZADA: AtomicBool=AtomicBool::new(false);

// Arquivos independentes editáveis em Inkscape, Figma e editores de texto.
pub const PENA_ORIGINAL:&str=include_str!("../assets/logos/pena.svg");
pub const SHEET_SVG:&str=include_str!("../assets/logos/feathersheet.svg");
pub const DOCS_SVG:&str=include_str!("../assets/logos/featherdocs.svg");
pub const SLIDE_SVG:&str=include_str!("../assets/logos/featherslide.svg");

pub fn definir_marca(produto:Produto, personalizada:bool) {
    PRODUTO.store(produto as u8,Ordering::Relaxed);
    COR_PERSONALIZADA.store(personalizada,Ordering::Relaxed);
}
pub fn produto_atual()->Produto {
    match PRODUTO.load(Ordering::Relaxed) {
        1=>Produto::FeatherDocs,2=>Produto::FeatherSlide,_=>Produto::FeatherSheet
    }
}
/// Identificador do aplicativo no sistema: o nome do `.desktop` instalado
/// (`feathersheet.desktop`) e o `app_id` das janelas no Wayland.
pub fn id_aplicativo()->&'static str {
    match produto_atual() {
        Produto::FeatherSheet=>"feathersheet",
        Produto::FeatherDocs=>"featherdocs",
        Produto::FeatherSlide=>"featherslide",
    }
}

static ICONE_JANELA: std::sync::OnceLock<Icon>=std::sync::OnceLock::new();

/// Ícone das janelas do aplicativo (barra de título e de tarefas no Windows e
/// no X11). O aplicativo registra uma vez, no começo; vale para todas as janelas
/// abertas por [`preparar_janela`].
pub fn definir_icone_janela(icone:Icon) {
    let _=ICONE_JANELA.set(icone);
}

/// Completa os ajustes de toda janela do produto: `app_id` (no Linux é o que
/// liga a janela ao `.desktop` instalado, ao nome e ao ícone no painel e no
/// gerenciador de tarefas; sem ele a janela aparece como "Processos do Sistema")
/// e o ícone registrado por [`definir_icone_janela`].
pub fn preparar_janela(mut ajustes:Settings)->Settings {
    if ajustes.app_id.is_none() { ajustes.app_id=Some(id_aplicativo().to_owned()); }
    if ajustes.icon.is_none() { ajustes.icon=ICONE_JANELA.get().cloned(); }
    ajustes
}

fn clarear(cor:&str,valor:f32)->String {
    let bits=u32::from_str_radix(cor.trim_start_matches('#'),16).unwrap_or(0x287846);
    let f=valor.clamp(0.,1.);
    let canal=|shift| {
        let cor=((bits >> shift) & 0xff_u32) as f32;
        (cor+(255.-cor)*f).round().clamp(0.,255.) as u32
    };
    format!("#{:02x}{:02x}{:02x}",canal(16),canal(8),canal(0))
}
/// A cor de destaque só recolore a marca quando foi selecionada pelo usuário.
pub fn svg_para(produto:Produto,personalizada:bool)->String {
    if !personalizada {
        return match produto {
            Produto::FeatherSheet=>SHEET_SVG,
            Produto::FeatherDocs=>DOCS_SVG,
            Produto::FeatherSlide=>SLIDE_SVG,
        }.to_owned();
    }
    let cor=tema::acento_hex();
    PENA_ORIGINAL
        .replace("#287846",cor)
        .replace("#72aa86",&clarear(cor,0.40))
}
/// Só as penas, sem quadrado de fundo: o SVG é transparente e o pelicano o desenha sobre a barra.
pub fn icone_para(produto:Produto,personalizada:bool,tamanho:f32)->SvgView {
    pelicano::svg(crate::icones::carregar(svg_para(produto,personalizada))).width(tamanho).height(tamanho)
}
pub fn icone_atual(tamanho:f32)->SvgView {
    icone_para(produto_atual(),COR_PERSONALIZADA.load(Ordering::Relaxed),tamanho)
}
#[cfg(test)]
mod testes {
    use super::*;
    #[test]
    fn tres_produtos_preservam_svg_vetorial_e_cores_de_fabrica() {
        for (produto,cor) in [
            (Produto::FeatherSheet,"#287846"),
            (Produto::FeatherDocs,"#236ac7"),
            (Produto::FeatherSlide,"#d1a219"),
        ] {
            let svg=svg_para(produto,false);
            assert!(svg.contains(cor));
            // SVG editado pelo Inkscape tem atributos id/fill antes de d.
            // Verificar dois caminhos independentes sem depender dessa ordem.
            assert_eq!(svg.matches("<path ").count(), 2, "duas penas vetoriais");
            assert!(svg.contains("id=\"pena-principal\""));
            assert!(svg.contains("id=\"pena-secundaria\""));
            assert!(svg.matches(" d=\"").count() >= 2);
            assert!(!svg.contains("<image"));
            assert!(!svg.contains("base64"));
        }
    }
    #[test]
    fn acento_personalizado_recolore_sem_sobrescrever_marca_padrao() {
        let svg=svg_para(Produto::FeatherSlide,true);
        assert!(svg.contains(tema::acento_hex()));
        assert_ne!(svg,svg_para(Produto::FeatherSlide,false));
        assert!(svg_para(Produto::FeatherSheet,false).contains("#287846"));
    }
    #[test]
    fn logos_sem_quadrado_de_fundo() {
        for produto in [Produto::FeatherSheet,Produto::FeatherDocs,Produto::FeatherSlide] {
            for personalizada in [false,true] {
                let svg=svg_para(produto,personalizada);
                assert!(!svg.contains("<rect"),"{produto:?}: a logo é só as penas");
                assert!(!svg.contains("fundo"),"{produto:?}: sem o degradê do fundo");
            }
        }
    }
    #[test]
    fn svg_instanciavel_sem_png_ou_arquivo_de_rede() {
        let _:SvgView=icone_para(Produto::FeatherSheet,false,24.);
        let _:SvgView=icone_para(Produto::FeatherDocs,false,72.);
        assert_eq!(preparar_janela(Settings::default()).app_id.as_deref(),Some(id_aplicativo()));
    }
}
