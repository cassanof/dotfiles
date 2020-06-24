/* See LICENSE file for copyright and license details. */

/* appearance */
static const char *fonts[] = { "fixed" };
static const char dmenufont[]            = "fixed";
static const char normbordercolor[] = "#cccccc";
static const char normbgcolor[]     = "#cccccc";
static const char normfgcolor[]     = "#000000";
static const char selbordercolor[]  = "#0066ff";
static const char selbgcolor[]      = "#0066ff";
static const char selfgcolor[]      = "#000000";
static unsigned int borderpx        = 0;
static unsigned int snap            = 32;
static const int showbar            = 0;        /* 0 means no bar */
static const int topbar             = 0;        /* 0 means bottom bar */

/* tagging */
static const char *tags[] = { "web" };

static const Rule rules[] = {
    {0}
};

/* layout(s) */
static const float mfact     = 0.55; /* factor of master area size [0.05..0.95] */
static const int nmaster     = 1;    /* number of clients in master area */
static const int resizehints = 0;    /* 1 means respect size hints in tiled resizals */

static const Layout layouts[] = {
	/* symbol     arrange function */
	{ "[M]",      monocle },
};

/* key definitions */
#define MODKEY Mod1Mask

/* commands */
static char dmenumon[2] = "0"; /* component of dmenucmd, manipulated in spawn() */
static const char *dmenucmd[] = { "dmenu_run", "-m", dmenumon, "-fn", dmenufont, "-nb", normbgcolor, "-nf", normfgcolor, "-sb", selbgcolor, "-sf", selfgcolor, NULL };

static Key keys[] = {
    /*  modifier        key     function        argument */ \
    { MODKEY|ShiftMask, XK_c,   killclient,     {0} },
    { MODKEY,           XK_q,   killclient,     {0} }, \
    { MODKEY,           XK_F4,  killclient,     {0} }, \
};

/* button definitions */
/* click can be ClkLtSymbol, ClkStatusText, ClkWinTitle, ClkClientWin, or ClkRootWin */
static Button buttons[] = {
    { 0 }
};
