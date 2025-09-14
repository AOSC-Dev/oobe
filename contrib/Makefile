SOURCES := aosc-os-oobe-cli-launcher aosc-os-oobe-gui-launcher

POT := po/aosc-os-oobe-launcher.pot

$(POT):
	mkdir -pv $$(dirname $(POT))
	xgettext $(SOURCES) -o $@ \
		--language=Shell \
		--copyright-holder="AOSC OS Maintainers <maintainers@aosc.io>" \
		--package-name="aosc-os-oobe-launcher" \
		--package-version="0.1.0" \
		2>/dev/null

.PHONY: aosc-os-oobe-launcher.po

.PHONY: update-po
update-po: $(wildcard po/*.po)

po/%.po: $(POT)
	msgmerge -U $@ $(POT)
	touch $@

po/%.mo: po/%.po
	msgfmt -o $@ $<
