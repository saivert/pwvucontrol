#!/bin/bash
export BUILD_ROOT="$1"

# English
mkdir -p $BUILD_ROOT/locale-test/locale/en/LC_MESSAGES
cp $BUILD_ROOT/po/en/LC_MESSAGES/pwvucontrol.mo $BUILD_ROOT/locale-test/locale/en/LC_MESSAGES

# Norwegian
mkdir -p $BUILD_ROOT/locale-test/locale/nb/LC_MESSAGES
cp $BUILD_ROOT/po/nb/LC_MESSAGES/pwvucontrol.mo $BUILD_ROOT/locale-test/locale/nb/LC_MESSAGES

