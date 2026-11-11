"""
Feather Sans — gerador de fonte arredondada para o FeatherOffice.

Cada letra é desenhada como um "esqueleto" (linhas e arcos) e depois
recebe espessura com pontas redondas. O peso da fonte é só a espessura
do traço, e o itálico é uma inclinação do esqueleto. Assim, todas as
versões saem do mesmo desenho e ficam consistentes.

Uso:  python feather_sans.py  ->  gera os .ttf na pasta ./fonts
Requer: pip install fonttools shapely
"""
import math, os
from shapely.geometry import LineString, Point, Polygon
from shapely.ops import unary_union
from shapely.geometry.polygon import orient
from shapely import affinity
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

UPM = 1000
XH, CAP, ASC, DESC = 520, 710, 770, -210
FAMILY = "Feather Sans"
NARROW = 0.9   # 1.0 = largura original; menor = mais estreita
VERSION = "1.001"

# ---------------------------------------------------------------- helpers
def arc(cx, cy, rx, ry, a0, a1, n=None):
    if n is None:
        n = max(6, int(abs(a1 - a0) / 3))
    out = []
    for i in range(n + 1):
        t = math.radians(a0 + (a1 - a0) * i / n)
        out.append((cx + rx * math.cos(t), cy + ry * math.sin(t)))
    return out

def ell(cx, cy, rx, ry):
    return arc(cx, cy, rx, ry, 0, 360, 120)


def build(w):
    """Devolve {caractere: (traços, pontos)} para uma espessura w."""
    h = w / 2
    B, X, C, A, D = h, XH - h, CAP - h, ASC - h, DESC + h
    ov = 7                                  # compensação óptica das curvas
    xm, cm = (B + X) / 2, (B + C) / 2
    H = C - B
    R = 232 + 0.12 * w                      # meia-largura do "o"
    ry = (X - B) / 2 + ov
    a = R * 0.86                            # meia-largura do arco do "n"
    ary = (X - B) * 0.52
    dr = h * 1.12 + 6                       # raio dos pingos
    doty = XH + 58 + dr + w * 0.15
    g = {}

    def S(*strokes, dots=()):
        return ([s if (s and s[0] in ('w', 'f')) else list(s) for s in strokes], list(dots))

    bowl = ell(R, xm, R, ry)
    # ---------------- minúsculas
    g['o'] = S(bowl)
    g['a'] = S(bowl, [(2*R, X), (2*R, B)])
    g['b'] = S([(0, A), (0, B)], bowl)
    g['d'] = S(bowl, [(2*R, A), (2*R, B)])
    g['p'] = S([(0, X), (0, D)], bowl)
    g['q'] = S(bowl, [(2*R, X), (2*R, D)])
    rh = 0.6 * (B - D) + 8
    rxh = min(R * 0.95, rh * 1.45)
    g['g'] = S(bowl, [(2*R, X), (2*R, D + rh)] + arc(2*R - rxh, D + rh, rxh, rh, 0, -158))
    g['c'] = S(arc(R, xm, R, ry, 50, 310))
    g['e'] = S([(0, xm), (2*R, xm)], arc(R, xm, R, ry, 0, 320))
    arch = lambda x0, ww: [(x0, X - ary)] + arc(x0 + ww, X - ary, ww, ary, 180, 0) + [(x0 + 2*ww, B)]
    g['n'] = S([(0, X), (0, B)], arch(0, a))
    a2 = a * 0.76
    g['m'] = S([(0, X), (0, B)], arch(0, a2), arch(2*a2, a2))
    g['h'] = S([(0, A), (0, B)], arch(0, a))
    g['u'] = S([(0, X), (0, B + ary)] + arc(a, B + ary, a, ary, 180, 360) + [(2*a, X)],
               [(2*a, X), (2*a, B)])
    g['r'] = S([(0, X), (0, B)], [(0, X - ary)] + arc(a*0.78, X - ary, a*0.78, ary, 180, 72))
    g['ı'] = S([(0, X), (0, B)])
    g['i'] = S([(0, X), (0, B)], dots=[(0, doty, dr)])
    rj = 0.5 * (B - D)
    jst = [(0, X), (0, D + rj)] + arc(-rj, D + rj, rj, rj, 0, -115)
    g['ȷ'] = S(jst)
    g['j'] = S(jst, dots=[(0, doty, dr)])
    rl = 95
    g['l'] = S([(0, A), (0, B + rl)] + arc(rl, B + rl, rl, rl, 180, 300))
    g['t'] = S([(0, X + 150), (0, B + rl)] + arc(rl, B + rl, rl, rl, 180, 300),
               [(-115, X), (165, X)])
    rf = 120
    g['f'] = S([(0, B), (0, A - rf)] + arc(rf, A - rf, rf, rf, 180, 30),
               [(-110, X), (175, X)])
    kw = 2 * a * 0.9
    yj = B + 0.36 * (X - B)
    kp = (kw * 0.42, X + 0.58 * (yj - X))
    g['k'] = S([(0, A), (0, B)], [(kw, X), (0, yj)], [kp, (kw, B)])
    vw = 2 * a * 1.02
    g['v'] = S([(0, X), (vw/2, B), (vw, X)])
    ww = 2 * a * 1.6
    g['w'] = S([(0, X), (ww*.25, B), (ww*.5, B + 0.85*(X-B)), (ww*.75, B), (ww, X)])
    xw = 2 * a * 0.95
    g['x'] = S([(0, X), (xw, B)], [(xw, X), (0, B)])
    yw = 2 * a * 1.02
    g['y'] = S([(0, X), (yw/2, B)], [(yw, X), (yw - (yw/2) * (X - D) / (X - B), D)])
    zw = 2 * a * 0.9
    g['z'] = S([(0, X), (zw, X), (0, B), (zw, B)])

    def s_shape(top, bot, sx):
        Ht = (top + ov) - (bot - ov)
        r1, r2 = 0.235 * Ht, 0.265 * Ht
        yt, yb = top + ov - r1, bot - ov + r2
        return arc(sx, yt, sx*0.93, r1, 28, 270) + arc(sx, yb, sx, r2, 90, -152)
    g['s'] = S(s_shape(X, B, R * 0.78))

    # ---------------- maiúsculas
    aw = 0.86 * H
    yb_ = B + 0.3 * H
    g['A'] = S([(0, B), (aw/2, C), (aw, B)],
               [(aw/2 * 0.3, yb_), (aw - aw/2 * 0.3, yb_)])
    r2, r1 = 0.27 * H, 0.23 * H
    x0t, x0b = 0.27 * H, 0.29 * H
    g['B'] = S([(0, C), (0, B)],
               [(0, C), (x0t, C)] + arc(x0t, C - r1, r1*1.05, r1, 90, -90) + [(0, C - 2*r1)],
               [(0, B + 2*r2), (x0b, B + 2*r2)] + arc(x0b, B + r2, r2*1.05, r2, 90, -90) + [(0, B)])
    rxC = 0.48 * H
    g['C'] = S(arc(rxC, cm, rxC, H/2 + ov, 45, 315))
    x0, rxD = 0.20 * H, 0.50 * H
    g['D'] = S([(0, C), (0, B)], [(0, C), (x0, C)] + arc(x0, cm, rxD, H/2, 90, -90) + [(0, B)])
    ew = 0.52 * H
    g['E'] = S([(ew, C), (0, C), (0, B), (ew, B)], [(0, B + 0.51*H), (ew*0.9, B + 0.51*H)])
    g['F'] = S([(ew, C), (0, C), (0, B)], [(0, B + 0.49*H), (ew*0.88, B + 0.49*H)])
    rG = 0.5 * H
    rqG = 0.11 * H
    ge = (rG + rG*math.cos(math.radians(-12)), cm + (H/2 + ov)*math.sin(math.radians(-12)))
    g['G'] = S(arc(rG, cm, rG, H/2 + ov, 45, 348) + arc(ge[0] - rqG, ge[1], rqG, rqG, 0, 90)
               + [(rG * 1.12, ge[1] + rqG)])
    hw = 0.62 * H
    g['H'] = S([(0, C), (0, B)], [(hw, C), (hw, B)], [(0, cm), (hw, cm)])
    g['I'] = S([(0, C), (0, B)])
    rJ = 0.25 * H
    g['J'] = S([(2*rJ, C), (2*rJ, B + rJ)] + arc(rJ, B + rJ, rJ, rJ, 0, -165))
    kW = 0.58 * H
    yjK = B + 0.38 * H
    kpK = (kW * 0.45, C + 0.55 * (yjK - C))
    g['K'] = S([(0, C), (0, B)], [(kW, C), (0, yjK)], [kpK, (kW, B)])
    g['L'] = S([(0, C), (0, B), (0.5*H, B)])
    mw = 0.84 * H
    g['M'] = S([(0, B), (0, C), (mw/2, B + 0.12*H), (mw, C), (mw, B)])
    nw = 0.64 * H
    g['N'] = S([(0, B), (0, C), (nw, B), (nw, C)])
    rO = 0.52 * H
    g['O'] = S(ell(rO, cm, rO, H/2 + ov))
    rP, x0P = 0.27 * H, 0.27 * H
    pb = [(0, C), (x0P, C)] + arc(x0P, C - rP, rP*1.05, rP, 90, -90) + [(0, C - 2*rP)]
    g['P'] = S([(0, C), (0, B)], pb)
    g['Q'] = S(ell(rO, cm, rO, H/2 + ov), [(rO*1.3, B + 0.18*H), (2*rO + 0.02*H, B - 0.06*H)])
    g['R'] = S([(0, C), (0, B)], pb, [(x0P*0.9, C - 2*rP), (x0P + rP*1.05, B)])
    g['S'] = S(s_shape(C, B, 0.4 * H))
    tw = 0.62 * H
    g['T'] = S([(0, C), (tw, C)], [(tw/2, C), (tw/2, B)])
    rU = 0.32 * H
    g['U'] = S([(0, C), (0, B + rU)] + arc(rU, B + rU, rU, rU, 180, 360) + [(2*rU, C)])
    VW = 0.8 * H
    g['V'] = S([(0, C), (VW/2, B), (VW, C)])
    WW = 1.18 * H
    g['W'] = S([(0, C), (WW*.25, B), (WW*.5, B + 0.82*H), (WW*.75, B), (WW, C)])
    XW = 0.7 * H
    g['X'] = S([(0, C), (XW, B)], [(XW, C), (0, B)])
    YW = 0.74 * H
    g['Y'] = S([(0, C), (YW/2, B + 0.45*H), (YW, C)], [(YW/2, B + 0.45*H), (YW/2, B)])
    ZW = 0.62 * H
    g['Z'] = S([(0, C), (ZW, C), (0, B), (ZW, B)])

    # ---------------- números
    g['0'] = S(ell(0.36*H, cm, 0.36*H, H/2 + ov))
    g['1'] = S([(-0.18*H, C - 0.16*H), (0, C), (0, B)])
    r = 0.3 * H
    g['2'] = S(arc(r, C - r, r, r, 160, -32) + [(0, B), (2*r*1.03, B)])
    t1, t2 = 0.23 * H, 0.27 * H
    g['3'] = S(arc(t2, C - t1, t1*1.08, t1, 150, -90) + arc(t2, B + t2, t2*1.08, t2, 90, -152))
    w4 = 0.68 * H
    g['4'] = S([(w4*0.72, B), (w4*0.72, C), (0, B + 0.3*H), (w4, B + 0.3*H)])
    r5 = 0.3 * H
    c5 = r5 + 0.02 * H
    st = (c5 + r5 * math.cos(math.radians(140)), B + r5 + r5 * math.sin(math.radians(140)))
    g['5'] = S([(c5 + r5*0.95, C), (st[0], C), st] + arc(c5, B + r5, r5, r5, 140, -150))
    r6 = 0.3 * H
    six = [ell(r6, B + r6, r6, r6), arc(0.5*H, B + r6, 0.5*H, C - (B + r6), 180, 76)]
    g['6'] = S(*six)
    W6 = 2 * r6
    g['9'] = S(*[[(W6 - x, B + C - y) for x, y in s] for s in six])
    g['7'] = S([(0, C), (0.6*H, C), (0.2*H, B)])
    g['8'] = S(ell(0.31*H, C - 0.22*H, 0.26*H, 0.22*H), ell(0.31*H, B + 0.28*H, 0.31*H, 0.28*H))

    # ---------------- pontuação
    dd = (0, dr, dr)
    comma = [(dr*0.25, dr*1.0), (-dr*0.45, -dr*1.5)]
    g['.'] = S(dots=[dd])
    g[','] = S(comma, dots=[dd])
    g[':'] = S(dots=[dd, (0, XH - dr, dr)])
    g[';'] = S(comma, dots=[dd, (0, XH - dr, dr)])
    g['!'] = S([(0, C), (0, B + 0.32*H)], dots=[dd])
    rq = 0.24 * H
    g['?'] = S(arc(rq, C - rq, rq*1.05, rq, 165, -90) + [(rq, B + 0.3*H)], dots=[(rq, dr, dr)])
    g['-'] = S([(0, xm), (0.3*H, xm)])
    g['–'] = S([(0, xm), (500 - w, xm)])
    g['—'] = S([(0, xm), (900 - w, xm)])
    g["'"] = S([(0, C), (0, C - 0.22*H)])
    g['"'] = S([(0, C), (0, C - 0.22*H)], [(0.17*H, C), (0.17*H, C - 0.22*H)])
    sh = CAP - 2*dr
    rq_ = [[(x, y + sh) for x, y in comma]], [(0, dr + sh, dr)]
    g['’'] = S(*rq_[0], dots=rq_[1])
    lq = [[(-x, 2*sh + 2*dr - y - sh + sh*0) for x, y in s] for s in rq_[0]]
    lq = [[(-x, CAP + (CAP - 2*dr) - y) for x, y in s] for s in rq_[0]]
    g['‘'] = S(*[[(-x, 2*CAP - 2*dr - y) for x, y in s] for s in rq_[0]], dots=[(0, CAP - dr, dr)])
    g['”'] = S(*rq_[0], [(x + 0.2*H, y) for x, y in rq_[0][0]], dots=[rq_[1][0], (0.2*H, dr + sh, dr)])
    g['“'] = S(*[[(-x, 2*CAP - 2*dr - y) for x, y in s] for s in rq_[0]],
               *[[(-x + 0.2*H, 2*CAP - 2*dr - y) for x, y in s] for s in rq_[0]],
               dots=[(0, CAP - dr, dr), (0.2*H, CAP - dr, dr)])
    g['…'] = S(dots=[dd, (dr*2 + 90, dr, dr), (dr*4 + 180, dr, dr)])
    g['•'] = S(dots=[(0, xm, dr * 1.5)])
    pm = (A + D) / 2
    pry = (A - D) / 2 / math.sin(math.radians(62))
    g['('] = S(arc(330, pm, 330, pry, 118, 242))
    g[')'] = S(arc(-330, pm, 330, pry, 62, -62))
    g['['] = S([(130, A), (0, A), (0, D), (130, D)])
    g[']'] = S([(-130, A), (0, A), (0, D), (-130, D)])
    g['/'] = S([(0, D), (0.42*H, A)])
    g['\\'] = S([(0.42*H, D), (0, A)])
    mY, p = 0.46 * CAP, 0.2 * H
    g['+'] = S([(-p, mY), (p, mY)], [(0, mY - p), (0, mY + p)])
    ig = max(0.1*H, (w + 45) / 2)           # no Bold as duas linhas não grudam
    g['='] = S([(-p, mY + ig), (p, mY + ig)], [(-p, mY - ig), (p, mY - ig)])
    g['<'] = S([(p, mY + p*0.9), (-p, mY), (p, mY - p*0.9)])
    g['>'] = S([(-p, mY + p*0.9), (p, mY), (-p, mY - p*0.9)])
    g['_'] = S([(0, DESC * 0.45), (0.5*H, DESC * 0.45)])
    ac, ar = C - 0.15*H, 0.13*H
    g['*'] = S(*[[(ar*math.cos(math.radians(t)), ac + ar*math.sin(math.radians(t))),
                  (-ar*math.cos(math.radians(t)), ac - ar*math.sin(math.radians(t)))]
                 for t in (90, 30, 150)])
    pr = 0.14 * H
    g['%'] = S(ell(pr, C - pr, pr, pr), ell(0.62*H, B + pr, pr, pr), [(0.62*H + pr*0.4, C), (pr - pr*0.4, B)])
    g['#'] = S([(0.2*H, C), (0.12*H, B)], [(0.48*H, C), (0.4*H, B)],
               [(0, B + 0.66*H), (0.62*H, B + 0.66*H)], [(-0.04*H, B + 0.33*H), (0.58*H, B + 0.33*H)])
    sx = 0.4 * H
    g['$'] = S(s_shape(C - 0.06*H, B + 0.06*H, sx), [(sx, C + 0.08*H), (sx, B - 0.08*H)])
    ri = 0.17 * H
    cy_ = B + 0.4 * H
    ss = 0.07 * H
    p0 = (ri + 2*ss, cy_ - ri*0.4)
    r2_ = math.hypot(p0[0], p0[1] - cy_)
    th0 = math.degrees(math.atan2(p0[1] - cy_, p0[0]))
    g['@'] = S(ell(0, cy_, ri, ri),
               [(ri, cy_ + ri), (ri, cy_ - ri*0.4)] + arc(ri + ss, cy_ - ri*0.4, ss, ss*1.2, 180, 360)
               + arc(0, cy_, r2_ * 1.9, r2_ * 1.9 * 1.05, th0 + 15, th0 + 300))
    g['°'] = S(ell(0, C - 0.12*H, 0.12*H, 0.12*H))

    # ---------------- símbolos extras
    def tf(glyph, sc, dx, dy, fac):
        st, dt = glyph
        out = []
        for s_ in st:
            pts = s_[2] if s_[0] == 'w' else s_
            out.append(('w', fac, [(x*sc + dx, y*sc + dy) for x, y in pts]))
        return out, [(x*sc + dx, y*sc + dy, r*sc*1.25) for x, y, r in dt]
    sup_fac = 0.72
    # &
    tcx, tcy, tr = 0.24*H, C - 0.17*H, 0.15*H
    bcx, bcy, br = 0.25*H, B + 0.25*H, 0.25*H
    pa = lambda cx, cy, rx, ry, t: (cx + rx*math.cos(math.radians(t)), cy + ry*math.sin(math.radians(t)))
    tcx, tcy, tr = 0.25*H, C - 0.16*H, 0.15*H
    bcx, bcy, br = 0.27*H, B + 0.26*H, 0.26*H
    pa = lambda cx, cy, rx, ry, t: (cx + rx*math.cos(math.radians(t)), cy + ry*math.sin(math.radians(t)))
    g['&'] = S(ell(tcx, tcy, tr*1.05, tr),
               [pa(tcx, tcy, tr*1.05, tr, 240), (0.74*H, B)],
               [pa(tcx, tcy, tr*1.05, tr, -50)] + arc(bcx, bcy, br*1.1, br, 140, 368) + [(0.84*H, B + 0.42*H)])
    # { } |
    rb = 55
    def brace(sign):
        p = [(130, A)] + arc(130 - rb, A - rb, rb, rb, 90, 180) + [(130 - 2*rb, pm + rb)] \
            + arc(130 - 3*rb, pm + rb, rb, rb, 0, -90) \
            + arc(130 - 3*rb, pm - rb, rb, rb, 90, 0) + [(130 - 2*rb, D + rb)] \
            + arc(130 - rb, D + rb, rb, rb, 180, 270)
        return [(x*sign, y) for x, y in p]
    g['{'] = S(brace(1))
    g['}'] = S(brace(-1))
    g['|'] = S([(0, A), (0, D)])
    g['~'] = S([(-0.2*H + 0.4*H*t/30, mY + 0.06*H*math.sin(2*math.pi*t/30)) for t in range(31)])
    g['^'] = S([(0, C - 0.3*H), (0.2*H, C), (0.4*H, C - 0.3*H)])
    g['`'] = S([(55, C - 30), (-45, C + 95)])
    g['´'] = S([(-45, C - 30), (55, C + 95)])
    g['¨'] = S(dots=[(-75, C + 30, dr), (75, C + 30, dr)])
    # ¡ ¿ (girados)
    def flip(glyph, cx, cy, dy):
        st, dt = glyph
        return ([[(2*cx - x, 2*cy - y + dy) for x, y in s_] for s_ in st],
                [(2*cx - x, 2*cy - y + dy, r) for x, y, r in dt])
    g['¡'] = flip(g['!'], 0, cm, -(C - X))
    g['¿'] = flip(g['?'], rq, cm, -(C - X))
    # moedas
    rE = 0.46*H
    g['€'] = S(arc(rE + 0.08*H, cm, rE, H/2 + ov, 50, 310),
               [(-0.04*H, cm + 0.1*H), (0.5*H, cm + 0.1*H)], [(-0.04*H, cm - 0.1*H), (0.45*H, cm - 0.1*H)])
    g['¢'] = S(arc(R*0.9, xm, R*0.9, ry, 50, 310), [(R*0.9, X + 0.12*H), (R*0.9, B - 0.12*H)])
    g['£'] = S(arc(0.36*H, C - 0.24*H, 0.22*H, 0.24*H, 25, 180) + [(0.14*H, B + 0.12*H), (0.0, B), (0.6*H, B)],
               [(0.0, cm), (0.4*H, cm)])
    g['¥'] = S([(0, C), (YW/2, B + 0.45*H), (YW, C)], [(YW/2, B + 0.45*H), (YW/2, B)],
               [(YW*0.18, B + 0.33*H), (YW*0.82, B + 0.33*H)], [(YW*0.18, B + 0.15*H), (YW*0.82, B + 0.15*H)])
    # © ®  ™
    rc_ = 0.5*H
    g['©'] = S(ell(rc_, cm, rc_, rc_), ('w', 0.8, arc(rc_, cm, 0.22*H, 0.24*H, 45, 315)))
    g['®'] = S(ell(rc_, cm, rc_, rc_),
               ('w', 0.8, [(rc_ - 0.16*H, cm - 0.25*H), (rc_ - 0.16*H, cm + 0.25*H), (rc_ + 0.02*H, cm + 0.25*H)]
                + arc(rc_ + 0.02*H, cm + 0.12*H, 0.14*H, 0.13*H, 90, -90) + [(rc_ - 0.16*H, cm - 0.01*H)]),
               ('w', 0.8, [(rc_ + 0.0*H, cm - 0.01*H), (rc_ + 0.17*H, cm - 0.25*H)]))
    tm = 0.42
    tT, _ = tf(g['T'], tm, 0, CAP*(1 - tm), sup_fac)
    tM, _ = tf(g['M'], tm, 0.62*H*tm + 0.12*H, CAP*(1 - tm), sup_fac)
    g['™'] = S(*tT, *tM)
    # × ÷ ±
    q = 0.17*H
    g['×'] = S([(-q, mY - q), (q, mY + q)], [(-q, mY + q), (q, mY - q)])
    dv = h + dr*0.75 + 34
    g['÷'] = S([(-p, mY), (p, mY)], dots=[(0, mY + dv, dr*0.75), (0, mY - dv, dr*0.75)])
    g['·'] = S(dots=[(0, xm, dr)])
    g['−'] = S([(-p, mY), (p, mY)])
    g['±'] = S([(-p, mY + 0.08*H), (p, mY + 0.08*H)], [(0, mY + 0.08*H - p), (0, mY + 0.08*H + p)],
               [(-p, mY - p - 0.06*H), (p, mY - p - 0.06*H)])
    # « »
    cq = 0.13*H
    chev = lambda x0, d: [(x0 + d*cq, xm + cq), (x0, xm), (x0 + d*cq, xm - cq)]
    g['«'] = S(chev(0, 1), chev(0.17*H, 1))
    g['»'] = S(chev(0, -1), chev(0.17*H, -1))
    g['‹'] = S(chev(0, 1))
    g['›'] = S(chev(0, -1))
    # ª º ¹ ² ³ (sobrescritos)
    ss_ = 0.5
    up = CAP - XH*ss_
    oa, da = tf(g['a'], ss_, 0, up, sup_fac)
    g['ª'] = S(*oa, ('w', sup_fac, [(0, up - 0.1*H), (2*R*ss_, up - 0.1*H)]), dots=da)
    oo, do_ = tf(g['o'], ss_, 0, up, sup_fac)
    g['º'] = S(*oo, ('w', sup_fac, [(0, up - 0.1*H), (2*R*ss_, up - 0.1*H)]), dots=do_)
    for d_, ch_ in (('1', '¹'), ('2', '²'), ('3', '³')):
        sc_ = 0.55
        st_, dt_ = tf(g[d_], sc_, 0, CAP*(1 - sc_), sup_fac)
        g[ch_] = S(*st_, dots=dt_)
    # §
    sxx = 0.3*H
    mlp, r1s, rxs = 0.16*H, 0.17*H, 0.25*H
    g['§'] = S(arc(rxs, C - r1s, rxs*0.95, r1s, 25, 270), ell(rxs, cm, rxs, mlp),
               arc(rxs, B + r1s, rxs*0.95, r1s, 90, -155))
    # ---------------- setas, frações, matemática, grego e sinais da interface
    # ('f', pontos) = forma cheia (triângulos, quadrado); o resto é traço.
    k = 0.2 * H                                   # tamanho da ponta das setas

    def seta(p0, p1, dupla=False, larga=False):
        (x0, y0), (x1, y1) = p0, p1
        L = math.hypot(x1 - x0, y1 - y0)
        dx, dy = (x1 - x0) / L, (y1 - y0) / L
        nx_, ny_ = -dy, dx
        kk = k * (1.15 if larga else 1.0)
        def ponta(px, py, sx, sy):
            return [(px - kk*sx + kk*nx_, py - kk*sy + kk*ny_), (px, py), (px - kk*sx - kk*nx_, py - kk*sy - kk*ny_)]
        out = [ponta(x1, y1, dx, dy)]
        if dupla:
            out.append(ponta(x0, y0, -dx, -dy))
        if larga:
            e = max(0.085 * H, (w + 50) / 2)       # no Bold as duas hastes não grudam
            a0 = kk * 0.55 if dupla else 0
            for sgn in (1, -1):
                out.append([(x0 + dx*a0 + sgn*e*nx_, y0 + dy*a0 + sgn*e*ny_),
                            (x1 - dx*kk*0.55 + sgn*e*nx_, y1 - dy*kk*0.55 + sgn*e*ny_)])
        else:
            out.append([(x0, y0), (x1, y1)])
        return out
    Ls = 0.78 * H
    g['←'] = S(*seta((Ls, mY), (0, mY)))
    g['→'] = S(*seta((0, mY), (Ls, mY)))
    g['↑'] = S(*seta((0, B), (0, C)))
    g['↓'] = S(*seta((0, C), (0, B)))
    g['↔'] = S(*seta((0, mY), (Ls*1.15, mY), dupla=True))
    g['↕'] = S(*seta((0, B), (0, C), dupla=True))
    g['⇐'] = S(*seta((Ls, mY), (0, mY), larga=True))
    g['⇒'] = S(*seta((0, mY), (Ls, mY), larga=True))
    g['⇔'] = S(*seta((0, mY), (Ls*1.2, mY), dupla=True, larga=True))
    dg = 0.55 * H
    g['↗'] = S(*seta((0, B), (dg, B + dg)))
    g['↘'] = S(*seta((0, B + dg), (dg, B)))
    g['↖'] = S(*seta((dg, B), (0, B + dg)))
    g['↙'] = S(*seta((dg, B + dg), (0, B)))

    # frações: numerador no alto à esquerda, denominador embaixo à direita
    def fracao(n, d):
        sc = 0.5
        sn, dn = tf(g[n], sc, 0, C - sc*C, sup_fac)
        sd, dd_ = tf(g[d], sc, 0.56*H, B - sc*B, sup_fac)
        return S(*sn, *sd, ('w', sup_fac, [(0.08*H, B), (0.72*H, C)]), dots=dn + dd_)
    for ch_, (n, d) in {'½': '12', '⅓': '13', '⅔': '23', '¼': '14', '¾': '34', '⅕': '15',
                        '⅛': '18', '⅜': '38', '⅝': '58', '⅞': '78'}.items():
        g[ch_] = fracao(n, d)

    # sobrescritos e subscritos
    for d_, ch_ in (('0', '⁰'), ('4', '⁴'), ('n', 'ⁿ')):
        sc_ = 0.55
        top = C if d_ != 'n' else X
        st_, dt_ = tf(g[d_], sc_, 0, CAP - top*sc_ - (CAP - C), sup_fac)
        g[ch_] = S(*st_, dots=dt_)
    for d_, ch_ in (('0', '₀'), ('1', '₁'), ('2', '₂'), ('3', '₃')):
        sc_ = 0.55
        st_, dt_ = tf(g[d_], sc_, 0, -0.14*CAP - sc_*B, sup_fac)
        g[ch_] = S(*st_, dots=dt_)

    # comparação e operadores
    g['≠'] = S(*g['='][0], [(-0.45*p, mY - 0.8*p), (0.45*p, mY + 0.8*p)])
    onda = lambda y: [(-0.2*H + 0.4*H*t/30, y + 0.06*H*math.sin(2*math.pi*t/30)) for t in range(31)]
    ao = max(0.1*H, (0.12*H + w + 45) / 2)
    g['≈'] = S(onda(mY + ao), onda(mY - ao))
    eq = max(0.15*H, w + 45)
    g['≡'] = S(*[[(-p, mY + dy_), (p, mY + dy_)] for dy_ in (eq, 0, -eq)])
    cy_, hh = mY + 0.09*H, p*0.72
    yl = cy_ - hh - max(0.14*H, w + 40)
    g['≤'] = S([(p, cy_ + hh), (-p, cy_), (p, cy_ - hh)], [(-p, yl), (p, yl)])
    g['≥'] = S([(-p, cy_ + hh), (p, cy_), (-p, cy_ - hh)], [(-p, yl), (p, yl)])
    def lemn(a_, cx, cy, t0=0, t1=360, n=90):
        out = []
        for i in range(n + 1):
            t = math.radians(t0 + (t1 - t0) * i / n)
            dn = 1 + math.sin(t)**2
            out.append((cx + a_*math.cos(t)/dn, cy + 1.35*a_*math.sin(t)*math.cos(t)/dn))
        return out
    ai = 0.38*H
    g['∞'] = S(lemn(ai, ai, mY))
    lado = lemn(ai*0.8, ai*0.8, mY, 90, 270, 60)
    g['∝'] = S(lado, [(ai*0.8, mY), (ai*1.65, mY + 0.17*H)], [(ai*0.8, mY), (ai*1.65, mY - 0.17*H)])
    raiz = [(0, B + 0.4*H), (0.1*H, B + 0.46*H), (0.28*H, B), (0.56*H, C + 0.06*H), (0.98*H, C + 0.06*H)]
    g['√'] = S(raiz)
    s3, d3 = tf(g['3'], 0.32, -0.05*H, B + 0.56*H - 0.32*B, sup_fac)
    g['∛'] = S(raiz, *s3, dots=d3)
    sw_ = 0.62*H
    sig = lambda y0, y1, sw: [(sw, y1), (0, y1), (sw*0.5, (y0 + y1)/2), (0, y0), (sw, y0)]
    g['Σ'] = S(sig(B, C, sw_))
    g['∑'] = S(sig(D + 0.05*H, A, sw_*1.1))
    pw = 0.72*H
    g['∏'] = S([(0, A), (pw, A)], [(0.1*pw, A), (0.1*pw, D + 0.05*H)], [(0.9*pw, A), (0.9*pw, D + 0.05*H)])
    ri_ = 0.12*H
    g['∫'] = S(arc(ri_, A - ri_, ri_, ri_, 30, 180) + [(0, D + ri_)] + arc(-ri_, D + ri_, ri_, ri_, 0, -150))
    g['∂'] = S(ell(0.85*R, xm, 0.85*R, ry), arc(0.75*R, xm, 0.95*R, C - xm, 0, 125))
    tri = lambda bw, y0, y1: [(0, y0), (bw/2, y1), (bw, y0), (0, y0)]
    g['Δ'] = S(tri(0.86*H, B, C))
    g['∆'] = g['Δ']
    g['∇'] = S(tri(0.86*H, C, B))

    # conjuntos e lógica
    rs = max(0.21*H, 1.45*w + 25)            # no Bold a barra do ∈ não encosta no arco
    abre = lambda cy: [(2*rs, cy + rs)] + arc(rs, cy, rs, rs, 90, 270) + [(2*rs, cy - rs)]
    g['∈'] = S(abre(mY), [(0, mY), (1.9*rs, mY)])
    g['∉'] = S(abre(mY), [(0, mY), (1.9*rs, mY)], [(0.35*rs, mY - 1.45*rs), (1.65*rs, mY + 1.45*rs)])
    g['⊂'] = S(abre(mY))
    yl2 = mY + 0.09*H - rs - max(0.14*H, w + 40)
    g['⊆'] = S(abre(mY + 0.09*H), [(0, yl2), (2*rs, yl2)])
    g['∪'] = S([(0, mY + rs*1.1), (0, mY)] + arc(rs, mY, rs, rs*0.9, 180, 360) + [(2*rs, mY + rs*1.1)])
    g['∩'] = S([(0, mY - rs*1.1), (0, mY)] + arc(rs, mY, rs, rs*0.9, 180, 0) + [(2*rs, mY - rs*1.1)])
    re_ = 0.3*H
    g['∅'] = S(ell(re_, cm, re_, re_), [(-0.04*H, B - 0.04*H), (2*re_ + 0.04*H, C + 0.04*H)])
    fw = 0.62*H
    yb_f = cm - 0.04*H
    g['∀'] = S([(0, C), (fw/2, B), (fw, C)], [(fw/2*(C - yb_f)/H, yb_f), (fw - fw/2*(C - yb_f)/H, yb_f)])
    g['∃'] = S([(0, C), (0.5*H, C), (0.5*H, B), (0, B)], [(0.06*H, cm), (0.5*H, cm)])
    g['¬'] = S([(-p, mY + 0.07*H), (p, mY + 0.07*H), (p, mY - 0.13*H)])
    g['∧'] = S([(-p, mY - p), (0, mY + p), (p, mY - p)])
    g['∨'] = S([(-p, mY + p), (0, mY - p), (p, mY + p)])
    dsx = 0.78*w + 40                            # traço interno das letras de quadro-negro
    g['ℕ'] = S(*g['N'][0], ('w', 0.55, [(dsx, B), (dsx, C - H*dsx/nw)]))
    g['ℤ'] = S(*g['Z'][0], ('w', 0.55, [(ZW - dsx, C), (0, B + H*dsx/ZW)]))
    g['ℝ'] = S(*g['R'][0], ('w', 0.55, [(dsx, B), (dsx, C)]))

    # grego minúsculo
    g['α'] = S(bowl, [(2*R + 0.13*H, X + 0.06*H), (2*R - 0.02*H, xm), (2*R + 0.15*H, B - 0.02*H)])
    tb = 0.21*H
    yb2 = A - 2*tb
    rb2 = (yb2 - B) / 2
    g['β'] = S([(0, D), (0, A - tb)] + arc(tb, A - tb, tb, tb, 180, -90) + [(0.12*H, yb2)]
               + arc(0.26*H, B + rb2, 0.26*H, rb2, 90, -90) + [(0, B + 0.08*H)])
    vg = 2*a*1.02
    g['γ'] = S([(0, X), (vg/2, B - 0.04*H), (vg/2, D + 0.05*H)], [(vg, X), (vg/2, B - 0.04*H)])
    g['δ'] = S(bowl, [(R*1.15, X + ov - 4), (R*0.45, X + 0.2*H), (R*0.55, A), (R*1.55, A)])
    g['ε'] = S(arc(0.8*R, (X + xm)/2, 0.72*R, (X - xm)/2 + ov, 55, 270)
               + arc(0.8*R, (xm + B)/2, 0.82*R, (xm - B)/2 + ov, 90, 305))
    rt_ = 0.3*H
    g['θ'] = S(ell(rt_, (A + B)/2, rt_, (A - B)/2 + ov), [(0, (A + B)/2), (2*rt_, (A + B)/2)])
    lw_ = 0.62*H
    yk = xm + 0.08*H
    xk = 0.06*H + (lw_ - 0.06*H) * (A - yk) / (A - B)
    g['λ'] = S([(0.06*H, A), (lw_, B)], [(xk, yk), (0, B)])
    g['µ'] = S([(0, X), (0, D)], *g['u'][0])
    g['μ'] = g['µ']
    pw2 = 2*a*1.12
    g['π'] = S([(-0.04*H, X), (pw2, X)], [(0.2*pw2, X), (0.14*pw2, B)], [(0.8*pw2, X), (0.8*pw2, B)])
    g['ρ'] = S(bowl, [(0, xm), (0, D)])
    g['σ'] = S(bowl, [(R, X + ov), (2*R + 0.2*H, X + ov)])
    g['τ'] = S([(0, X), (2*a, X)], [(a, X), (a, B + rl)] + arc(a + rl, B + rl, rl, rl, 180, 300))
    g['φ'] = S(bowl, [(R, X + 0.16*H), (R, D)])
    w2 = a*0.82
    g['ω'] = S([(0, X), (0, B + ary)] + arc(w2, B + ary, w2, ary, 180, 360) + [(2*w2, xm)],
               [(2*w2, B + ary)] + arc(3*w2, B + ary, w2, ary, 180, 360) + [(4*w2, X)])
    # grego maiúsculo
    rO2 = 0.44*H
    cyO = cm + 0.08*H
    g['Ω'] = S([(0, B), (0.62*rO2, B)] + arc(rO2, cyO, rO2, 0.44*H, 235, -55) + [(1.38*rO2, B), (2*rO2, B)])

    # moedas que faltavam
    g['₩'] = S(*g['W'][0], [(-0.06*H, B + 0.6*H), (WW + 0.06*H, B + 0.6*H)],
               [(-0.02*H, B + 0.36*H), (WW + 0.02*H, B + 0.36*H)])
    g['₽'] = S(*g['P'][0], [(-0.16*H, B + 0.22*H), (0.42*H, B + 0.22*H)], [(-0.16*H, C - 2*rP), (0, C - 2*rP)])
    g['₿'] = S(*g['B'][0], [(0.1*H, C), (0.1*H, C + 0.13*H)], [(0.3*H, C), (0.3*H, C + 0.13*H)],
               [(0.1*H, B), (0.1*H, B - 0.13*H)], [(0.3*H, B), (0.3*H, B - 0.13*H)])
    g['₹'] = S([(0, C), (0.58*H, C)], [(0, C - 0.18*H), (0.58*H, C - 0.18*H)],
               [(0, C), (0.18*H, C)] + arc(0.18*H, C - 0.24*H, 0.22*H, 0.24*H, 90, -90) + [(0, C - 0.48*H), (0.46*H, B)])

    # pontuação de texto
    g['¶'] = S([(0.36*H, C), (0.36*H, B)], [(0.58*H, C), (0.58*H, B)],
               [(0.58*H, C), (0.24*H, C)] + arc(0.24*H, C - 0.2*H, 0.2*H, 0.2*H, 90, 270) + [(0.36*H, C - 0.4*H)])
    g['†'] = S([(0, C), (0, D + 0.1*H)], [(-0.2*H, C - 0.24*H), (0.2*H, C - 0.24*H)])
    g['‡'] = S([(0, C), (0, D + 0.1*H)], [(-0.2*H, C - 0.24*H), (0.2*H, C - 0.24*H)],
               [(-0.2*H, B + 0.06*H), (0.2*H, B + 0.06*H)])
    g['‰'] = S(*g['%'][0], ell(0.62*H + 2*pr + 0.08*H, B + pr, pr, pr))

    # sinais da interface: triângulos, bolinhas, quadrados, certo e errado
    ts = 0.2*H
    g['▾'] = S(('f', [(0, mY + 0.55*ts), (2*ts, mY + 0.55*ts), (ts, mY - 0.65*ts)]))
    g['▴'] = S(('f', [(0, mY - 0.55*ts), (2*ts, mY - 0.55*ts), (ts, mY + 0.65*ts)]))
    g['▸'] = S(('f', [(0, mY + ts), (0, mY - ts), (1.2*ts, mY)]))
    g['◂'] = S(('f', [(1.2*ts, mY + ts), (1.2*ts, mY - ts), (0, mY)]))
    tg = 0.4*H
    g['▲'] = S(('f', [(0, cm - 0.86*tg), (2*tg, cm - 0.86*tg), (tg, cm + 0.86*tg)]))
    g['▼'] = S(('f', [(0, cm + 0.86*tg), (2*tg, cm + 0.86*tg), (tg, cm - 0.86*tg)]))
    g['►'] = S(('f', [(0, cm + tg), (0, cm - tg), (1.7*tg, cm)]))
    g['◄'] = S(('f', [(1.7*tg, cm + tg), (1.7*tg, cm - tg), (0, cm)]))
    rc2 = 0.3*H
    g['●'] = S(dots=[(rc2, mY, rc2)])
    g['○'] = S(ell(rc2, mY, rc2 - h, rc2 - h))
    q2 = 0.3*H
    g['■'] = S(('f', [(0, mY - q2), (2*q2, mY - q2), (2*q2, mY + q2), (0, mY + q2)]))
    g['□'] = S([(0, mY - q2), (2*q2, mY - q2), (2*q2, mY + q2), (0, mY + q2), (0, mY - q2)])
    g['✓'] = S([(0, mY + 0.02*H), (0.22*H, mY - 0.26*H), (0.64*H, mY + 0.42*H)])
    g['✗'] = S([(0.04*H, mY + 0.34*H), (0.52*H, mY - 0.32*H)], [(0.5*H, mY + 0.36*H), (0, mY - 0.3*H)])
    g['ø'] = S(bowl, [(0.12*R, B - 0.06*H), (1.88*R, X + 0.06*H)])
    g['Ø'] = S(ell(rO, cm, rO, H/2 + ov), [(0.2*rO, B - 0.04*H), (1.8*rO, C + 0.04*H)])
    return g


import unicodedata
MARKS = {'0301': 'acute', '0300': 'grave', '0302': 'circ', '0303': 'tilde', '0308': 'diaer', '0327': 'cedilla'}
COMPOSE = {}
for ch in 'áàâãäéèêëíìîïóòôõöúùûüñÁÀÂÃÄÉÈÊËÍÌÎÏÓÒÔÕÖÚÙÛÜÑ':
    base_hex, mark = unicodedata.decomposition(ch).split()
    base = chr(int(base_hex, 16))
    COMPOSE[ch] = ('ı' if base == 'i' else base, MARKS[mark])
COMPOSE['ç'] = ('c', 'cedilla')
COMPOSE['Ç'] = ('C', 'cedilla')


def accent(kind, cx, upper, w):
    h = w / 2
    dr = h * 1.12 + 6
    yb = (CAP + 45 if upper else XH + 55) + h
    if kind == 'acute':  return [[(cx - 45, yb), (cx + 55, yb + 125)]], []
    if kind == 'grave':  return [[(cx + 45, yb), (cx - 55, yb + 125)]], []
    if kind == 'circ':   return [[(cx - 95, yb), (cx, yb + 105), (cx + 95, yb)]], []
    if kind == 'tilde':
        return [[(cx - 105 + 210*t/30, yb + 40 + 34*math.sin(2*math.pi*t/30)) for t in range(31)]], []
    if kind == 'diaer':  return [], [(cx - 75, yb + dr - h, dr), (cx + 75, yb + dr - h, dr)]
    if kind == 'cedilla':
        rc = 46 + h * 0.55
        top = -h * 0.2
        return [[(cx, top + 10), (cx, top - 25)] + arc(cx, top - 25 - rc, rc * 1.15, rc, 90, -105)], []


def outline(strokes, dots, w):
    shapes = []
    for s in strokes:
        ww = w
        if s and s[0] == 'f':
            # forma cheia com cantos arredondados, como as pontas dos traços
            shapes.append(Polygon(s[1]).buffer(w * 0.18, quad_segs=16, join_style='round'))
            continue
        if s and s[0] == 'w':
            ww, s = w * s[1], s[2]
        if len(s) == 1:
            shapes.append(Point(s[0]).buffer(ww / 2, quad_segs=16))
            continue
        shapes.append(LineString(s).buffer(ww / 2, quad_segs=16, cap_style='round', join_style='round'))
    for x, y, r in dots:
        shapes.append(Point(x, y).buffer(r, quad_segs=16))
    return unary_union(shapes) if shapes else None



def kerning_fea():
    """Ajustes de espaço entre pares de letras (kerning)."""
    u = lambda ch: 'uni%04X' % ord(ch)
    cls = lambda chars: '[' + ' '.join(u(c) for c in chars) + ']'
    round_lc = 'aceogqdsçáàâãéêóôõ'
    pairs = [
        ('f', 'f', -105), ('f', 'tıíj', -40), ('t', 't', -60), ('f', round_lc, -30),
        ('T', round_lc + 'ruvwxyzmnp', -70), ('T', 'AÁÀÂÃ', -60), ('AÁÀÂÃ', 'T', -60),
        ('VW', round_lc, -45), ('VW', 'AÁÀÂÃ', -50), ('AÁÀÂÃ', 'VW', -50),
        ('Y', round_lc, -60), ('Y', 'AÁÀÂÃ', -55), ('AÁÀÂÃ', 'Y', -55),
        ('AÁÀÂÃ', 'vwy', -30), ('vwy', 'AÁÀÂÃ', -25),
        ('L', 'TVWY', -70), ('L', '’”\'"', -80),
        ('P', 'AÁÀÂÃ', -45), ('PFT', '.,…', -90), ('VWY', '.,…', -70),
        ('rvwy', '.,…', -45), ('r', round_lc, -12),
        ('k', round_lc, -15), ('x', round_lc, -12),
        ('LTVWY', '-–', -30), ('-–', 'TVWY', -30),
        ('“‘', 'AÁÀÂÃ', -50), ('AÁÀÂÃ', '’”', -50),
        ('F', round_lc, -25), ('J', '.,', -20),
    ]
    lines = ['languagesystem DFLT dflt;', 'languagesystem latn dflt;', 'feature kern {']
    for a_, b_, v in pairs:
        lines.append(f'  pos {cls(a_)} {cls(b_)} {v};')
    lines.append('} kern;')
    return '\n'.join(lines)

STRAIGHT_L = set('bhiklmnprBDEFHIKLMNPRuıȷ|')
STRAIGHT_R = set('adhimnqluHIMNıI|')


def make_font(name, style, w, italic, weight_class, outdir):
    slant = 10 if italic else 0
    sk = math.tan(math.radians(slant))
    g = build(w)
    sb = 40 + 0.12 * w
    glyphs, metrics, cmap = {}, {}, {}
    order = ['.notdef', 'space', 'nbspace']

    def add(ch, strokes, dots):
        nx = lambda pts: [(x * NARROW, y) for x, y in pts]
        strokes = [('w', s_[1], nx(s_[2])) if (s_ and s_[0] == 'w') else
                   ('f', nx(s_[1])) if (s_ and s_[0] == 'f') else nx(s_) for s_ in strokes]
        dots = [(x * NARROW, y, r) for x, y, r in dots]
        geom = outline(strokes, dots, w)
        minx, _, maxx, _ = geom.bounds
        l = sb + (12 if ch in STRAIGHT_L else 0)
        rr = sb + (12 if ch in STRAIGHT_R else 0)
        geom = affinity.translate(geom, l - minx, 0)
        adv = round(maxx - minx + l + rr)
        if slant:
            geom = affinity.skew(geom, xs=slant, origin=(0, XH / 2))
        pen = TTGlyphPen(None)
        polys = geom.geoms if hasattr(geom, 'geoms') else [geom]
        for poly in polys:
            poly = orient(poly.simplify(0.6, preserve_topology=True), sign=-1.0)
            for ring in [poly.exterior] + list(poly.interiors):
                pts = []
                for x, y in list(ring.coords)[:-1]:
                    p = (round(x), round(y))
                    if not pts or pts[-1] != p:
                        pts.append(p)
                if len(pts) > 1 and pts[0] == pts[-1]:
                    pts.pop()
                if len(pts) < 3:
                    continue
                pen.moveTo(pts[0])
                for p in pts[1:]:
                    pen.lineTo(p)
                pen.closePath()
        gname = 'uni%04X' % ord(ch)
        glyphs[gname] = pen.glyph()
        metrics[gname] = (adv, 0)
        cmap[ord(ch)] = gname
        order.append(gname)

    for ch, (st, dt) in g.items():
        add(ch, st, dt)
    for ch, (base, kind) in COMPOSE.items():
        st, dt = g[base]
        xs = [x for s in st for x, _ in s] + [x for x, _, _ in dt]
        cx = (min(xs) + max(xs)) / 2
        if base in 'ı':
            cx = 0
        ast, adt = accent(kind, cx, base.isupper(), w)
        add(ch, st + ast, dt + adt)

    # .notdef e espaços
    pen = TTGlyphPen(None)
    for ring in ([(50, 0), (50, 700), (450, 700), (450, 0)], [(100, 50), (400, 50), (400, 650), (100, 650)]):
        pen.moveTo(ring[0]); [pen.lineTo(p) for p in ring[1:]]; pen.closePath()
    glyphs['.notdef'] = pen.glyph(); metrics['.notdef'] = (500, 50)
    sp = round(250 + 0.3 * w)
    for n in ('space', 'nbspace'):
        glyphs[n] = TTGlyphPen(None).glyph(); metrics[n] = (sp, 0)
    cmap[0x20] = 'space'; cmap[0xA0] = 'nbspace'

    # lsb real
    fb = FontBuilder(UPM, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyphs)
    glyf = fb.font['glyf']
    for n in order:
        gl = glyf[n]
        gl.recalcBounds(glyf)
        metrics[n] = (metrics[n][0], getattr(gl, 'xMin', 0) if gl.numberOfContours else 0)
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=1000, descent=-260, lineGap=0,
                             caretSlopeRise=1000 if slant else 1,
                             caretSlopeRun=round(1000 * sk) if slant else 0)
    ribbi = style in ('Regular', 'Italic', 'Bold', 'Bold Italic')
    fam = FAMILY if ribbi else f"{FAMILY} {name}"
    sub = style if ribbi else ('Italic' if italic else 'Regular')
    full = f"{FAMILY} {style}"
    ps = (FAMILY + '-' + style).replace(' ', '')
    names = {'familyName': fam, 'styleName': sub, 'uniqueFontIdentifier': f"{ps};{VERSION}",
             'fullName': full, 'psName': ps, 'version': f"Version {VERSION}",
             'copyright': 'Feather Sans, feita para o FeatherOffice',
             'licenseDescription': 'SIL Open Font License 1.1'}
    if not ribbi:
        names['typographicFamily'] = FAMILY
        names['typographicSubfamily'] = style
    fb.setupNameTable(names)
    bold = weight_class >= 700
    fs = (1 if italic else 0) | (0x20 if bold else 0) | (0x40 if (not italic and not bold) else 0) 
    fb.setupOS2(usWeightClass=weight_class, fsSelection=fs, achVendID='FTHR',
                sTypoAscender=800, sTypoDescender=-200, sTypoLineGap=200,
                usWinAscent=1000, usWinDescent=260, sxHeight=XH, sCapHeight=CAP,
                fsType=0)
    fb.setupPost(italicAngle=-slant)
    fb.font['head'].macStyle = (1 if bold else 0) | (2 if italic else 0)
    os.makedirs(outdir, exist_ok=True)
    path = os.path.join(outdir, ps + '.ttf')
    from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
    addOpenTypeFeaturesFromString(fb.font, kerning_fea())
    fb.save(path)
    return path


STYLES = [  # (nome do peso, estilo, espessura, itálico, peso OS/2)
    ('Thin', 'Thin', 38, False, 200),
    ('Thin', 'Thin Italic', 38, True, 200),
    ('', 'Regular', 84, False, 400),
    ('', 'Italic', 84, True, 400),
    ('', 'Bold', 128, False, 700),
    ('', 'Bold Italic', 128, True, 700),
]

if __name__ == '__main__':
    for args in STYLES:
        print(make_font(*args, outdir='fonts'))
