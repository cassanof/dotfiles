# dwm-rs build and install helpers

CARGO ?= cargo
INSTALL ?= install
SUDO ?= sudo

PREFIX ?= /usr/local
BINDIR ?= ${PREFIX}/bin
XSESSIONSDIR ?= /usr/share/xsessions
SYSCONFDIR ?= /etc

TARGET ?= release
CRATE_BIN ?= dwm-rs
BIN ?= ${CRATE_BIN}
DESKTOP_ENTRY ?= dwm-rs.desktop
CONFIG_FILE ?= dwm.toml

CARGO_BUILD_FLAGS_debug =
CARGO_BUILD_FLAGS_release = --release
CARGO_BUILD_FLAGS = ${CARGO_BUILD_FLAGS_${TARGET}}

TARGET_DIR = target/${TARGET}
TARGET_BIN = ${TARGET_DIR}/${CRATE_BIN}
BUILD_DIR = .build
GENERATED_DESKTOP_ENTRY = ${BUILD_DIR}/${DESKTOP_ENTRY}

all:
	${CARGO} build ${CARGO_BUILD_FLAGS}

clean:
	${CARGO} clean
	rm -rf ${BUILD_DIR}

${GENERATED_DESKTOP_ENTRY}: ${DESKTOP_ENTRY}
	mkdir -p ${BUILD_DIR}
	sed \
		-e 's|^Name=.*|Name=${BIN}|' \
		-e 's|^Exec=.*|Exec=${BINDIR}/${BIN}|' \
		< ${DESKTOP_ENTRY} > ${GENERATED_DESKTOP_ENTRY}

install: all ${GENERATED_DESKTOP_ENTRY}
	${SUDO} ${INSTALL} -d ${DESTDIR}${BINDIR}
	${SUDO} ${INSTALL} -m 755 ${TARGET_BIN} ${DESTDIR}${BINDIR}/${BIN}
	${SUDO} ${INSTALL} -d ${DESTDIR}${XSESSIONSDIR}
	${SUDO} ${INSTALL} -m 644 ${GENERATED_DESKTOP_ENTRY} ${DESTDIR}${XSESSIONSDIR}/${DESKTOP_ENTRY}
	${SUDO} ${INSTALL} -d ${DESTDIR}${SYSCONFDIR}
	${SUDO} ${INSTALL} -m 644 ${CONFIG_FILE} ${DESTDIR}${SYSCONFDIR}/dwm.toml

uninstall:
	${SUDO} rm -f ${DESTDIR}${BINDIR}/${BIN}
	${SUDO} rm -f ${DESTDIR}${XSESSIONSDIR}/${DESKTOP_ENTRY}

.PHONY: all clean install uninstall
