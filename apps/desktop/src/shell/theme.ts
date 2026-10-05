// The Nexees look of the window (apps/desktop/src/shell/theme): the dark colour theme and the
// compact sizes of the approved design. Every colour and size here is a design token
// (design_tokens.ts, the Desktop copy of assets/theme/), so that Android can show the same
// product (R18, R19). This file only says which token each part of Theia's workbench takes.
//
// Nothing is drawn anew: the theme is a colour theme in the format Theia shares with VS Code,
// registered like any other, and the sizes are Theia's own style variables. The Nexees theme is
// the default: apps/desktop/package.json names it as `defaultTheme`, and its `preferences` object,
// though empty, is what makes Theia apply that default. Theia's other themes stay available to
// the user, and the sizes hold under every theme.

import { injectable, inject } from '@theia/core/shared/inversify';
import { FrontendApplicationContribution } from '@theia/core/lib/browser/frontend-application-contribution';
import { ColorTheme, CssStyleCollector, StylingParticipant } from '@theia/core/lib/browser/styling-service';
import { MonacoThemingService } from '@theia/monaco/lib/browser/monaco-theming-service';
import { TOKENS } from './design_tokens';

/** The ID of the Nexees colour theme, which apps/desktop/package.json names as the default. */
const NEXEES_DARK = 'nexees-dark';

const { surface, border, text, accent, status, syntax } = TOKENS.color;

/** `colour` at an opacity from 0 to 1, as #rrggbbaa. */
function alpha(colour: string, opacity: number): string {
    return colour + Math.round(opacity * 255).toString(16).padStart(2, '0');
}

/**
 * The workbench colours of the Nexees theme, by colour ID. Whatever is not named keeps the
 * default of Theia's dark themes.
 */
const WORKBENCH_COLOURS: Readonly<Record<string, string>> = {
    // Text and focus everywhere.
    'foreground': text.primary,
    'descriptionForeground': text.secondary,
    'disabledForeground': text.muted,
    'errorForeground': status.error,
    'icon.foreground': text.secondary,
    'focusBorder': accent.primary,
    'selection.background': alpha(accent.fill, 0.6),
    'widget.border': border.subtle,
    'textLink.foreground': accent.primary,
    'textLink.activeForeground': accent.primary,
    'textPreformat.foreground': text.primary,
    'textPreformat.background': surface.raised,
    'textBlockQuote.background': surface.raised,
    'textBlockQuote.border': border.subtle,
    'textCodeBlock.background': surface.raised,
    'textSeparator.foreground': border.subtle,
    'sash.hoverBorder': accent.primary,
    'toolbar.hoverBackground': surface.selected,
    'toolbar.activeBackground': surface.selected,

    // The title row and its menus.
    'titleBar.activeBackground': surface.window,
    'titleBar.activeForeground': text.primary,
    'titleBar.inactiveBackground': surface.window,
    'titleBar.inactiveForeground': text.muted,
    'titleBar.border': border.subtle,
    'menubar.selectionBackground': surface.selected,
    'menubar.selectionForeground': text.primary,
    'menu.background': surface.raised,
    'menu.foreground': text.primary,
    'menu.selectionBackground': accent.fill,
    'menu.selectionForeground': text.on_accent,
    'menu.separatorBackground': border.subtle,
    'menu.border': border.subtle,

    // The activity bar and the side bars.
    'activityBar.background': surface.window,
    'activityBar.foreground': accent.primary,
    'activityBar.inactiveForeground': text.secondary,
    'activityBar.activeBorder': accent.primary,
    'activityBar.activeBackground': surface.selected,
    'activityBar.border': border.subtle,
    'activityBarBadge.background': accent.fill,
    'activityBarBadge.foreground': text.on_accent,
    'activityErrorBadge.background': status.error,
    'activityErrorBadge.foreground': surface.window,
    'activityWarningBadge.background': status.warning,
    'activityWarningBadge.foreground': surface.window,
    'sideBar.background': surface.window,
    'sideBar.foreground': text.primary,
    'sideBarSectionHeader.background': surface.window,
    'sideBarSectionHeader.foreground': text.secondary,
    'sideBarSectionHeader.border': border.subtle,

    // Lists and trees.
    'list.activeSelectionBackground': surface.selected,
    'list.activeSelectionForeground': text.primary,
    'list.activeSelectionIconForeground': text.primary,
    'list.inactiveSelectionBackground': surface.selected,
    'list.inactiveSelectionForeground': text.primary,
    'list.hoverBackground': surface.hover,
    'list.hoverForeground': text.primary,
    'list.focusBackground': surface.selected,
    'list.focusForeground': text.primary,
    'list.focusOutline': accent.primary,
    'list.highlightForeground': accent.primary,
    'list.dropBackground': alpha(accent.primary, 0.2),
    'list.errorForeground': status.error,
    'list.warningForeground': status.warning,
    'list.invalidItemForeground': status.warning,
    'list.deemphasizedForeground': text.muted,
    'list.filterMatchBackground': alpha(accent.primary, 0.3),
    'listFilterWidget.background': surface.raised,
    'listFilterWidget.noMatchesOutline': status.error,
    'tree.indentGuidesStroke': border.subtle,

    // Editor groups and tabs.
    'editorGroup.border': border.subtle,
    'editorGroup.dropBackground': alpha(accent.primary, 0.18),
    'editorGroupHeader.tabsBackground': surface.window,
    'editorGroupHeader.tabsBorder': border.subtle,
    'tab.activeBackground': surface.selected,
    'tab.activeForeground': text.primary,
    'tab.activeBorder': accent.primary,
    'tab.inactiveBackground': surface.window,
    'tab.inactiveForeground': text.secondary,
    'tab.unfocusedActiveBackground': surface.selected,
    'tab.unfocusedActiveForeground': text.secondary,
    'tab.unfocusedInactiveForeground': text.muted,
    'tab.hoverBackground': surface.hover,
    'tab.border': surface.window,
    'tab.activeModifiedBorder': accent.primary,
    'tab.inactiveModifiedBorder': alpha(accent.primary, 0.5),
    'breadcrumb.background': surface.editor,
    'breadcrumb.foreground': text.secondary,
    'breadcrumb.focusForeground': text.primary,
    'breadcrumb.activeSelectionForeground': text.primary,
    'breadcrumbPicker.background': surface.raised,

    // The editor.
    'editor.background': surface.editor,
    'editor.foreground': text.primary,
    'editorCursor.foreground': accent.primary,
    'editorLineNumber.foreground': text.muted,
    'editorLineNumber.activeForeground': text.primary,
    'editor.lineHighlightBackground': surface.hover,
    'editor.lineHighlightBorder': surface.hover,
    'editor.selectionBackground': alpha(accent.fill, 0.45),
    'editor.inactiveSelectionBackground': alpha(accent.fill, 0.25),
    'editor.selectionHighlightBackground': alpha(accent.primary, 0.15),
    'editor.wordHighlightBackground': alpha(text.muted, 0.25),
    'editor.wordHighlightStrongBackground': alpha(accent.primary, 0.25),
    'editor.findMatchBackground': alpha(status.warning, 0.4),
    'editor.findMatchHighlightBackground': alpha(status.warning, 0.2),
    'editor.hoverHighlightBackground': alpha(accent.primary, 0.15),
    'editorBracketMatch.background': alpha(accent.primary, 0.15),
    'editorBracketMatch.border': alpha(accent.primary, 0.6),
    'editorIndentGuide.background1': border.subtle,
    'editorIndentGuide.activeBackground1': text.muted,
    'editorWhitespace.foreground': alpha(text.muted, 0.4),
    'editorRuler.foreground': border.subtle,
    'editorLink.activeForeground': accent.primary,
    'editorCodeLens.foreground': text.muted,
    'editor.foldPlaceholderForeground': text.muted,
    'editorLightBulb.foreground': status.warning,
    'editorLightBulbAutoFix.foreground': accent.primary,
    'editorLightBulbAi.foreground': accent.secondary,
    'editorError.foreground': status.error,
    'editorWarning.foreground': status.warning,
    'editorInfo.foreground': accent.primary,
    'editorGutter.background': surface.editor,
    'editorWidget.background': surface.raised,
    'editorWidget.foreground': text.primary,
    'editorWidget.border': border.subtle,
    'editorHoverWidget.background': surface.raised,
    'editorHoverWidget.foreground': text.primary,
    'editorHoverWidget.border': border.subtle,
    'editorSuggestWidget.background': surface.raised,
    'editorSuggestWidget.foreground': text.primary,
    'editorSuggestWidget.border': border.subtle,
    'editorSuggestWidget.selectedBackground': accent.fill,
    'editorSuggestWidget.selectedForeground': text.on_accent,
    'editorSuggestWidget.highlightForeground': accent.primary,
    'scrollbarSlider.background': alpha(text.muted, 0.25),
    'scrollbarSlider.hoverBackground': alpha(text.muted, 0.4),
    'scrollbarSlider.activeBackground': alpha(text.muted, 0.55),

    // The bottom panel and the terminal. The terminal's sixteen colours are the status, accent
    // and text colours, so that success, warning and error look the same there as everywhere.
    'panel.background': surface.window,
    'panel.border': border.subtle,
    'panelTitle.activeForeground': text.primary,
    'panelTitle.inactiveForeground': text.secondary,
    'panelTitle.activeBorder': accent.primary,
    'panelInput.border': border.subtle,
    'terminal.background': surface.window,
    'terminal.foreground': text.primary,
    'terminal.border': border.subtle,
    'terminal.selectionBackground': alpha(accent.fill, 0.45),
    'terminalCursor.foreground': accent.primary,
    'terminal.ansiBlack': surface.raised,
    'terminal.ansiRed': status.error,
    'terminal.ansiGreen': status.success,
    'terminal.ansiYellow': status.warning,
    'terminal.ansiBlue': accent.primary,
    'terminal.ansiMagenta': accent.secondary,
    'terminal.ansiCyan': syntax.type,
    'terminal.ansiWhite': text.secondary,
    'terminal.ansiBrightBlack': text.muted,
    'terminal.ansiBrightRed': status.error,
    'terminal.ansiBrightGreen': status.success,
    'terminal.ansiBrightYellow': status.warning,
    'terminal.ansiBrightBlue': accent.primary,
    'terminal.ansiBrightMagenta': accent.secondary,
    'terminal.ansiBrightCyan': syntax.type,
    'terminal.ansiBrightWhite': text.primary,

    // The status bar. An item that reports an error or a warning takes the status colour as
    // its background, with the window's dark surface as its text.
    'statusBar.background': surface.raised,
    'statusBar.foreground': text.secondary,
    'statusBar.border': border.subtle,
    'statusBar.noFolderBackground': surface.raised,
    'statusBar.noFolderForeground': text.secondary,
    'statusBar.offlineBackground': status.warning,
    'statusBar.offlineForeground': surface.window,
    'statusBarItem.hoverBackground': surface.selected,
    'statusBarItem.hoverForeground': text.primary,
    'statusBarItem.activeBackground': surface.selected,
    'statusBarItem.errorBackground': status.error,
    'statusBarItem.errorForeground': surface.window,
    'statusBarItem.warningBackground': status.warning,
    'statusBarItem.warningForeground': surface.window,
    'statusBarItem.prominentBackground': surface.selected,
    'statusBarItem.prominentForeground': text.primary,

    // Inputs, buttons and badges.
    'input.background': surface.raised,
    'input.foreground': text.primary,
    'input.border': border.subtle,
    'input.placeholderForeground': text.muted,
    'inputOption.activeBorder': accent.primary,
    'inputOption.activeBackground': alpha(accent.primary, 0.25),
    'inputOption.activeForeground': text.primary,
    'inputValidation.errorBackground': surface.raised,
    'inputValidation.errorBorder': status.error,
    'inputValidation.warningBackground': surface.raised,
    'inputValidation.warningBorder': status.warning,
    'inputValidation.infoBackground': surface.raised,
    'inputValidation.infoBorder': accent.primary,
    'dropdown.background': surface.raised,
    'dropdown.foreground': text.primary,
    'dropdown.border': border.subtle,
    'checkbox.background': surface.raised,
    'checkbox.foreground': text.primary,
    'checkbox.border': border.subtle,
    'button.background': accent.fill,
    'button.foreground': text.on_accent,
    'button.hoverBackground': accent.fill_hover,
    'button.secondaryBackground': surface.selected,
    'button.secondaryForeground': text.primary,
    'button.secondaryHoverBackground': border.subtle,
    'secondaryButton.background': surface.selected,
    'secondaryButton.foreground': text.primary,
    'secondaryButton.hoverBackground': border.subtle,
    'badge.background': accent.fill,
    'badge.foreground': text.on_accent,
    'progressBar.background': accent.primary,
    'keybindingLabel.background': surface.selected,
    'keybindingLabel.foreground': text.primary,
    'keybindingLabel.border': border.subtle,
    'keybindingLabel.bottomBorder': border.subtle,

    // Notifications, the command palette and the problem markers.
    'notifications.background': surface.raised,
    'notifications.foreground': text.primary,
    'notifications.border': border.subtle,
    'notificationCenterHeader.background': surface.selected,
    'notificationLink.foreground': accent.primary,
    'notificationsErrorIcon.foreground': status.error,
    'notificationsWarningIcon.foreground': status.warning,
    'notificationsInfoIcon.foreground': accent.primary,
    'quickInput.background': surface.raised,
    'quickInput.foreground': text.primary,
    'quickInputTitle.background': surface.selected,
    'quickInputList.focusBackground': accent.fill,
    'quickInputList.focusForeground': text.on_accent,
    'quickInputList.focusIconForeground': text.on_accent,
    'pickerGroup.border': border.subtle,
    'pickerGroup.foreground': accent.primary,
    'problemsErrorIcon.foreground': status.error,
    'problemsWarningIcon.foreground': status.warning,
    'problemsInfoIcon.foreground': accent.primary,
    'settings.headerForeground': text.primary,
    'settings.modifiedItemIndicator': accent.primary,
};

/** The colours of code, by TextMate scope. Code outside these scopes keeps the editor's text colour. */
const SYNTAX_COLOURS: ReadonlyArray<{ scope: string[]; settings: { foreground: string } }> = [
    { scope: ['comment', 'punctuation.definition.comment'], settings: { foreground: text.muted } },
    { scope: ['keyword', 'storage'], settings: { foreground: syntax.keyword } },
    { scope: ['string'], settings: { foreground: syntax.string } },
    { scope: ['constant.numeric', 'constant.language'], settings: { foreground: syntax.number } },
    { scope: ['entity.name.function', 'support.function'], settings: { foreground: syntax.function } },
    { scope: ['entity.name.type', 'entity.name.class', 'support.type', 'support.class'], settings: { foreground: syntax.type } },
    { scope: ['variable'], settings: { foreground: syntax.variable } },
];

/**
 * The sizes of the compact layout, as rules for the window's style sheet: Theia's own size
 * variables take the tokens, and its inputs, buttons and selection boxes take the Nexees corner
 * radius.
 */
function sizeRules(): string[] {
    const variables = {
        '--theia-ui-font-size1': TOKENS.font.base,
        '--theia-statusBar-font-size': TOKENS.font.small,
        '--theia-ui-padding': TOKENS.space.unit,
        '--theia-content-line-height': TOKENS.desktop.row_height,
        '--theia-private-horizontal-tab-height': TOKENS.desktop.tab_height,
        '--theia-statusBar-height': TOKENS.desktop.status_bar_height,
        '--theia-private-sidebar-tab-width': TOKENS.desktop.activity_bar_width,
        '--theia-private-menubar-height': TOKENS.desktop.title_bar_height,
    };
    const declarations = Object.entries(variables).map(([name, pixels]) => `${name}: ${pixels}px;`);
    return [
        `:root { ${declarations.join(' ')} }`,
        `.theia-input, .theia-button, .theia-select, .theia-select-component { border-radius: ${TOKENS.radius.control}px; }`,
    ];
}

/** Registers the Nexees colour theme with Theia and adds the size rules to the window's style. */
@injectable()
export class NexeesTheme implements FrontendApplicationContribution, StylingParticipant {

    constructor(@inject(MonacoThemingService) protected readonly theming: MonacoThemingService) { }

    initialize(): void {
        this.theming.registerParsedTheme({
            id: NEXEES_DARK,
            label: 'Nexees Dark',
            uiTheme: 'vs-dark',
            json: { name: 'Nexees Dark', colors: WORKBENCH_COLOURS, tokenColors: SYNTAX_COLOURS },
        });
    }

    registerThemeStyle(_theme: ColorTheme, collector: CssStyleCollector): void {
        sizeRules().forEach(rule => collector.addRule(rule));
    }
}
