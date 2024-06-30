#!/bin/bash
export BUILD_ROOT="$1"

mkdir -p $BUILD_ROOT/locale-test/
ln -s $BUILD_ROOT/po $BUILD_ROOT/locale-test/locale
