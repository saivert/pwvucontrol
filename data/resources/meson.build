blueprints = custom_target('blueprints',
  input: files(
    'ui/profile-dropdown.blp',
    'ui/volumescale.blp',
    'ui/volumebox.blp',
    'ui/window.blp',
    'ui/help-overlay.blp',
    'ui/sinkbox.blp',
    'ui/devicebox.blp',
    'ui/stream-dropdown.blp',
    'ui/channelbox.blp',
    'ui/streambox.blp',
    'ui/profilerow.blp',
    'ui/route-dropdown.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

# Resources
resources = gnome.compile_resources(
  'resources',
  'resources.gresource.xml',
  gresource_bundle: true,
  dependencies: blueprints,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: pkgdatadir,
)
