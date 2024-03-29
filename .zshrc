# If you come from bash you might have to change your $PATH.
# export PATH=$HOME/bin:/usr/local/bin:$PATH

# Path to your oh-my-zsh installation.
export ZSH="${HOME}/.oh-my-zsh"

# Set name of the theme to load --- if set to "random", it will
# load a random theme each time oh-my-zsh is loaded, in which case,
# to know which specific one was loaded, run: echo $RANDOM_THEME
# See https://github.com/ohmyzsh/ohmyzsh/wiki/Themes
ZSH_THEME="elleven"

# Set list of themes to pick from when loading at random
# Setting this variable when ZSH_THEME=random will cause zsh to load
# a theme from this variable instead of looking in ~/.oh-my-zsh/themes/
# If set to an empty array, this variable will have no effect.
# ZSH_THEME_RANDOM_CANDIDATES=( "robbyrussell" "agnoster" )

# Uncomment the following line to use case-sensitive completion.
# CASE_SENSITIVE="true"

# Uncomment the following line to use hyphen-insensitive completion.
# Case-sensitive completion must be off. _ and - will be interchangeable.
# HYPHEN_INSENSITIVE="true"

# Uncomment the following line to disable bi-weekly auto-update checks.
# DISABLE_AUTO_UPDATE="true"

# Uncomment the following line to automatically update without prompting.
DISABLE_UPDATE_PROMPT="true"

# Uncomment the following line to change how often to auto-update (in days).
export UPDATE_ZSH_DAYS=5

# Uncomment the following line if pasting URLs and other text is messed up.
# DISABLE_MAGIC_FUNCTIONS=true

# Uncomment the following line to disable colors in ls.
DISABLE_LS_COLORS="false"

# Uncomment the following line to disable auto-setting terminal title.
DISABLE_AUTO_TITLE="true"

# Sets the title to whatever is passed as $1
function set-term-title {
	# Escape the argument for printf formatting.
	local title=$1
	title=${title//\\/\\\\}
	title=${title//\"/\\\"}
	title=${title//\%/\%\%}

	# OSC 0, 1, and 2 are the portable escape codes for setting window titles.
	printf "\e]0;$title\a"  # Both tab and window
	printf "\e]1;$title\a"  # Tab title
	printf "\e]2;$title\a"  # Window title
}

# show full path in terminal title
function precmd () {
  set-term-title "$(print -P %~)"
}

function preexec {
  set-term-title "$(print -P %~# $1)"
}

function x11-clip-wrap-widgets() {
    # NB: Assume we are the first wrapper and that we only wrap native widgets
    # See zsh-autosuggestions.zsh for a more generic and more robust wrapper
    local copy_or_paste=$1
    shift

    for widget in $@; do
        # Ugh, zsh doesn't have closures
        if [[ $copy_or_paste == "copy" ]]; then
            eval "
            function _x11-clip-wrapped-$widget() {
                zle .$widget
                echo -n "\$CUTBUFFER" | xclip -in -selection clipboard
            }
            "
        else
            eval "
            function _x11-clip-wrapped-$widget() {
                CUTBUFFER=\$(xclip -out -selection clipboard)
                zle .$widget
            }
            "
        fi

        zle -N $widget _x11-clip-wrapped-$widget
    done
}


local copy_widgets=(
    vi-yank vi-yank-eol vi-delete vi-backward-kill-word vi-change-whole-line
)
local paste_widgets=(
    vi-put-{before,after}
)

# NB: can atm. only wrap native widgets
x11-clip-wrap-widgets copy $copy_widgets
x11-clip-wrap-widgets paste  $paste_widgets


# Uncomment the following line to enable command auto-correction.
ENABLE_CORRECTION="false"

# Uncomment the following line to display red dots whilst waiting for completion.
# COMPLETION_WAITING_DOTS="true"

# Uncomment the following line if you want to disable marking untracked files
# under VCS as dirty. This makes repository status check for large repositories
# much, much faster.
# DISABLE_UNTRACKED_FILES_DIRTY="true"

# Uncomment the following line if you want to change the command execution time
# stamp shown in the history command output.
# You can set one of the optional three formats:
# "mm/dd/yyyy"|"dd.mm.yyyy"|"yyyy-mm-dd"
# or set a custom format using the strftime function format specifications,
# see 'man strftime' for details.
# HIST_STAMPS="mm/dd/yyyy"

# Would you like to use another custom folder than $ZSH/custom?
# ZSH_CUSTOM=/path/to/new-custom-folder

# Which plugins would you like to load?
# Standard plugins can be found in ~/.oh-my-zsh/plugins/*
# Custom plugins may be added to ~/.oh-my-zsh/custom/plugins/
# Example format: plugins=(rails git textmate ruby lighthouse)
# Add wisely, as too many plugins slow down shell startup.
plugins=(
        git 
        zsh-autosuggestions 
        zsh-syntax-highlighting
        vi-mode
        )

source $ZSH/oh-my-zsh.sh
source $HOME/.env/bin/activate

# User configuration

# export MANPATH="/usr/local/man:$MANPATH"

# You may need to manually set your language environment
# export LANG=en_US.UTF-8

# Preferred editor for local and remote sessions
export EDITOR='vim'

# Preferred browser
export BROWSER='librewolf'

# Compilation flags
# export ARCHFLAGS="-arch x86_64"

# Set personal aliases, overriding those provided by oh-my-zsh libs,
# plugins, and themes. Aliases can be placed here, though oh-my-zsh
# users are encouraged to define aliases within the ZSH_CUSTOM folder.
# For a full list of active aliases, run `alias`.
#
# Example aliases
# alias zshconfig="mate ~/.zshrc"
# alias ohmyzsh="mate ~/.oh-my-zsh"

# ==================== #
#        PATHS         #
# ==================== #

# nvm path (node version manager)
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

# go paths
export PATH="$PATH:$HOME/go/bin"  # global go bins
export GOPATH="$HOME/go"

# chrome exec to chromium
export CHROME_EXECUTABLE=/usr/bin/chromium

# flutter path
export PATH="$PATH:$HOME/code/flutter/bin"

# clang-format path
export PATH="/opt/clang-format-static:$PATH"

# local bins path
export PATH="/home/elleven/.local/bin/:$PATH"

# ruby bins path
export PATH="/home/elleven/.local/share/gem/ruby/3.0.0/bin/:$PATH"

# cargo bin
export PATH="/home/elleven/.cargo/bin/:$PATH"

# classpath
export CLASSPATH="/home/elleven/.classpath/*:$CLASSPATH"

# neovim + latex + zathura stuff
export NVIM_LISTEN_ADDRESS=/tmp/nvimsocket

# ocaml
eval $(opam env)

# ==================== #
#       ALIASES        #
# ==================== #
LS_COLORS='di=1;34:fi=37:ln=31:pi=5:so=5:bd=5:cd=5:or=31:mi=0:ex=33;100:*.png=35:*.gif=36:*.jpg=35:*.c=92:*.vim=92:*.java=92:*.go=92:*.jar=33:*.rs=92:*.py=92:*.h=90:*.txt=94:*.md=7;34:*.doc=7;34:*.docx=7;34:*.odt=7;34:*.csv=7;32:*.xlsx=7;32:*.xlsm=7;32:*.rb=31:*.cpp=92:*.sh=96:*.css=90:*.json=90:*.js=92:*.ts=92:*.html=96:*.pug=96:*.cfg=96:*.conf=96:*.config=96:*.zip=4;33:*.gz=4;33:*.xz=4;33:*.mp4=105:*.mp3=106'
export LS_OPTIONS='--color=auto'
alias ls='ls $LS_OPTIONS -F'
alias ll='ls $LS_OPTIONS -Flah'
alias l='ls $LS_OPTIONS -Fa'

# cpu governor aliases
alias powersave='sudo cpupower frequency-set --governor powersave'
alias performance='sudo cpupower frequency-set --governor performance'

# shortcut for sudo pacman
alias pac='sudo pacman'

# shortcut for python venvs
alias virtualenv2='python2 -m virtualenv'
alias virtualenv3='python3 -m virtualenv'

# shortcut for disabling and enabling aslr
alias aslr_off='echo 0 | sudo tee /proc/sys/kernel/randomize_va_space'
alias aslr_on='echo 2 | sudo tee /proc/sys/kernel/randomize_va_space'

# shortcut for msfconsole with sigtrap
alias msfc='/home/elleven/code/dotfiles/scripts/msfconsole_sigtrap.sh --quiet'

# script to lookup monero price
alias xmrprice="curl 'https://min-api.cryptocompare.com/data/price?fsym=XMR&tsyms=BTC,USD,EUR'"

# script to find any ip in stream
alias findips="grep -oE '\b([0-9]{1,3}\.){3}[0-9]{1,3}\b'"

# alias to volatility
alias volatility3="python3 /home/elleven/cybersec/tools/forensics/volatility3/vol.py"
alias volatility2="/home/elleven/cybersec/tools/forensics/volatility3/vol2"

# alias for jupyter lab to open with librewolf
alias jlab="jupyter lab --browser=librewolf"

# alias for send2trash
alias tm="send2trash -v"

# alias for pandoc
alias pandock=\
'docker run --rm -v "$(pwd):/data" -u $(id -u):$(id -g) pandoc/latex'

# alias to see files in a dir
alias imgdir="feh -g 640x480 -d -S filename"

# use doas
alias sudo='doas'
alias sudoedit='doas rnano'

# colored man pages
export LESS_TERMCAP_mb=$'\e[1;32m'
export LESS_TERMCAP_md=$'\e[1;32m'
export LESS_TERMCAP_me=$'\e[0m'
export LESS_TERMCAP_se=$'\e[0m'
export LESS_TERMCAP_so=$'\e[01;33m'
export LESS_TERMCAP_ue=$'\e[0m'
export LESS_TERMCAP_us=$'\e[1;4;31m'

# if there is no line under here, the install script wasn't used or something went wrong
export DOTFILESDIR=/home/elleven/code/dotfiles
