desktop_file = i18n.merge_file(
        input: 'com.saivert.pwvucontrol.desktop.in',
       output: 'com.saivert.pwvucontrol.desktop',
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
        input: 'com.saivert.pwvucontrol.appdata.xml.in',
       output: 'com.saivert.pwvucontrol.appdata.xml',
       po_dir: '../po',
      install: true,
  install_dir: join_paths(get_option('datadir'), 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util, args: ['validate', appstream_file])
endif

install_data('com.saivert.pwvucontrol.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file',
       compile_schemas,
       args: ['--strict', '--dry-run', meson.current_source_dir()])
endif

dev_schema = custom_target(
  'dev-schema',
  build_by_default: true,
  build_always_stale: true,
  input: 'com.saivert.pwvucontrol.gschema.xml',
  output: 'gschemas.compiled',
  console: true,
  install: false,
  command: [
    'env', compile_schemas,
    '--strict', meson.current_source_dir(),
  ]
)

subdir('icons')
