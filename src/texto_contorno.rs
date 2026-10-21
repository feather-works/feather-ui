//! Contorno das letras pela fonte do pelicano: o texto vira polígonos (um por subcaminho), com os
//! miolos preservados (O, B, a, e…). Serve ao texto artístico (WordArt) de qualquer aplicativo:
//! quem chama deforma os pontos (por exemplo com `feather_shapes::wordart::transformar`) e
//! desenha os polígonos com preenchimento par/ímpar ou não nulo.
//!
//! Sem dependência de motor: a deformação entra como função, para o `feather-ui` não puxar o
//! `feather-shapes`.
use pelicano::{Font, Path, Point, Renderer, Segment};

/// Tolerância do achatamento das curvas da fonte, em pixels.
const TOLERANCIA: f32 = 0.12;

/// Resolve a fonte do `texto` e devolve todos os subcaminhos como polígonos (`[x, y]` lógicos;
/// `em` é o topo-esquerdo da primeira linha). Subcaminho com menos de 3 pontos sai.
pub fn contornos(r: &mut Renderer, texto: &str, tamanho: f32, fonte: Font, em: Point) -> Vec<Vec<[f32; 2]>> {
    r.text_path(texto, tamanho, fonte, em, None).map(|p| poligonos(&p)).unwrap_or_default()
}

/// [`contornos`] com cada ponto passado por `deformar` (a transformação do WordArt).
pub fn contornos_deformados(
    r: &mut Renderer,
    texto: &str,
    tamanho: f32,
    fonte: Font,
    em: Point,
    deformar: impl Fn([f32; 2]) -> [f32; 2],
) -> Vec<Vec<[f32; 2]>> {
    contornos(r, texto, tamanho, fonte, em)
        .into_iter()
        .map(|p| p.into_iter().map(&deformar).collect())
        .collect()
}

/// Os subcaminhos de um `Path` achatados em polígonos.
pub fn poligonos(caminho: &Path) -> Vec<Vec<[f32; 2]>> {
    let mut fora = Vec::new();
    let mut atual: Vec<[f32; 2]> = Vec::new();
    let fechar = |atual: &mut Vec<[f32; 2]>, fora: &mut Vec<Vec<[f32; 2]>>| {
        if atual.len() > 2 {
            fora.push(std::mem::take(atual));
        }
        atual.clear();
    };
    let xy = |p: Point| [p.x, p.y];
    for seg in caminho.segments() {
        let a = atual.last().copied().unwrap_or_default();
        match seg {
            Segment::MoveTo(p) => {
                fechar(&mut atual, &mut fora);
                atual.push(xy(p));
            }
            Segment::LineTo(p) => atual.push(xy(p)),
            Segment::QuadTo(c, b) => {
                let (c, b) = (xy(c), xy(b));
                // Erro da corda ≤ |a - 2c + b| / (4n²).
                let n = passos(desvio(a, c, b) / 4.0);
                atual.extend((1..=n).map(|i| {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    let k = [u * u, 2.0 * u * t, t * t];
                    [k[0] * a[0] + k[1] * c[0] + k[2] * b[0], k[0] * a[1] + k[1] * c[1] + k[2] * b[1]]
                }));
            }
            Segment::CubicTo(c1, c2, b) => {
                let (c1, c2, b) = (xy(c1), xy(c2), xy(b));
                // Erro da corda ≤ 0,75 · max|segunda diferença| / n².
                let n = passos(0.75 * desvio(a, c1, c2).max(desvio(c1, c2, b)));
                atual.extend((1..=n).map(|i| {
                    let t = i as f32 / n as f32;
                    let u = 1.0 - t;
                    let k = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                    [
                        k[0] * a[0] + k[1] * c1[0] + k[2] * c2[0] + k[3] * b[0],
                        k[0] * a[1] + k[1] * c1[1] + k[2] * c2[1] + k[3] * b[1],
                    ]
                }));
            }
            Segment::Close => fechar(&mut atual, &mut fora),
        }
    }
    fechar(&mut atual, &mut fora);
    fora
}

/// |a - 2b + c|: quanto a curva se afasta da corda.
fn desvio(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (a[0] - 2.0 * b[0] + c[0]).hypot(a[1] - 2.0 * b[1] + c[1])
}

/// Quantos segmentos deixam o erro `e / n²` abaixo da tolerância.
fn passos(e: f32) -> usize {
    ((e / TOLERANCIA).sqrt().ceil() as usize).clamp(1, 100)
}

/// Caixa `[x, y, largura, altura]` que envolve os polígonos; `None` se não houver pontos.
pub fn caixa(poligonos: &[Vec<[f32; 2]>]) -> Option<[f32; 4]> {
    let mut it = poligonos.iter().flatten();
    let &[x0, y0] = it.next()?;
    let (mut a, mut b, mut c, mut d) = (x0, y0, x0, y0);
    for &[x, y] in it {
        a = a.min(x);
        b = b.min(y);
        c = c.max(x);
        d = d.max(y);
    }
    Some([a, b, c - a, d - b])
}

#[cfg(test)]
mod testes {
    use super::*;

    fn contornos_de(texto: &str) -> Vec<Vec<[f32; 2]>> {
        contornos(&mut Renderer::new(), texto, 48.0, Font::DEFAULT, Point::new(10.0, 10.0))
    }

    #[test]
    fn letra_o_tem_miolo() {
        let c = contornos_de("O");
        assert!(c.len() >= 2, "O precisa de contorno externo e interno, veio {}", c.len());
        let [x, y, w, h] = caixa(&c).unwrap();
        assert!(x >= 9.0 && y >= 9.0 && w > 10.0 && h > 10.0);
    }

    #[test]
    fn deformar_desloca_todos_os_pontos() {
        let mut r = Renderer::new();
        let base = contornos(&mut r, "A", 48.0, Font::DEFAULT, Point::new(10.0, 10.0));
        let movidos = contornos_deformados(&mut r, "A", 48.0, Font::DEFAULT, Point::new(10.0, 10.0), |[x, y]| [x + 5.0, y]);
        assert_eq!(base.len(), movidos.len());
        assert_eq!(movidos[0][0], [base[0][0][0] + 5.0, base[0][0][1]]);
        assert!(contornos_de("").is_empty());
        assert_eq!(caixa(&[]), None);
    }

    #[test]
    fn caminho_com_curvas_vira_poligonos() {
        let p = Path::new(|b| {
            b.move_to(Point::new(0.0, 0.0)).line_to(Point::new(10.0, 0.0)).quadratic_to(Point::new(10.0, 10.0), Point::new(0.0, 10.0)).close();
            b.move_to(Point::new(20.0, 0.0)).bezier_to(Point::new(30.0, 0.0), Point::new(30.0, 10.0), Point::new(20.0, 10.0)).close();
        })
        .unwrap();
        let c = poligonos(&p);
        assert_eq!(c.len(), 2);
        assert!(c[0].len() > 4 && c[1].len() > 4);
        assert_eq!(c[0][..2], [[0.0, 0.0], [10.0, 0.0]]);
        assert_eq!(*c[1].last().unwrap(), [20.0, 10.0]);
    }
}
