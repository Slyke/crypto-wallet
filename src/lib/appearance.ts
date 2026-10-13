export const palettes = ['d4rk', 'terminal', 'paper', 'ocean'] as const;
export type Palette = (typeof palettes)[number];
export interface Appearance {
    palette: Palette;
    theme: 'dark' | 'light';
    font: string;
}
export const readAppearance = (): Appearance => {
    const result: Appearance = { palette: 'd4rk', theme: 'dark', font: 'system-stack' };
    try {
        const palette = localStorage.getItem('d4rk-styleguide-palette');
        const theme = localStorage.getItem('d4rk-styleguide-theme');
        const font = localStorage.getItem('d4rk-styleguide-font');
        if (palettes.includes(palette as Palette)) result.palette = palette as Palette;
        if (theme === 'dark' || theme === 'light') result.theme = theme;
        if (font && ['system-stack', 'consolas', 'cascadia-mono'].includes(font)) result.font = font;
    } catch {
        /* A denied preference store must not prevent using the app. */
    }
    return result;
};
export const applyAppearance = ({ appearance }: { appearance: Appearance }) => {
    Object.assign(document.documentElement.dataset, appearance);
    try {
        for (const key of ['palette', 'theme', 'font'] as const) localStorage.setItem('d4rk-styleguide-' + key, appearance[key]);
    } catch {
        /* Preferences are optional; no vault data ever enters this store. */
    }
};
