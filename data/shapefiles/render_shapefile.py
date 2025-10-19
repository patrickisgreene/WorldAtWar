"""
# Basic render
python scripts/render_shapefile.py countries.shp world.png

# High resolution with custom colors
python scripts/render_shapefile.py coastlines.shp coast.png --width 4096 --height 2048 --bg-color "#0a1628" --fill-color "#FFFFFF"

# With outlines
python scripts/render_shapefile.py borders.shp borders.png --fill-color "#CCCCCC" --line-color "#000000" --line-width 2

# Mercator projection
python scripts/render_shapefile.py data.shp output.png --projection mercator
"""

import sys
import os
from PIL import Image, ImageDraw
import shapefile
import numpy as np

def get_bounds(shapes):
    """Calculate bounding box for all shapes."""
    min_x = float('inf')
    min_y = float('inf')
    max_x = float('-inf')
    max_y = float('-inf')

    for shape in shapes:
        for point in shape.points:
            min_x = min(min_x, point[0])
            max_x = max(max_x, point[0])
            min_y = min(min_y, point[1])
            max_y = max(max_y, point[1])

    return min_x, min_y, max_x, max_y

def project_point(lon, lat, min_lon, min_lat, max_lon, max_lat, width, height, projection='equirectangular'):
    """Project geographic coordinates to pixel coordinates."""
    if projection == 'equirectangular':
        # Simple equirectangular projection
        x = (lon - min_lon) / (max_lon - min_lon) * width
        y = (max_lat - lat) / (max_lat - min_lat) * height  # Flip Y axis
        return int(x), int(y)
    elif projection == 'mercator':
        # Web Mercator projection
        from math import log, tan, pi, radians
        x = (lon - min_lon) / (max_lon - min_lon) * width

        # Mercator Y
        lat_rad = radians(lat)
        merc_y = log(tan(pi / 4 + lat_rad / 2))

        min_lat_rad = radians(min_lat)
        max_lat_rad = radians(max_lat)
        min_merc_y = log(tan(pi / 4 + min_lat_rad / 2))
        max_merc_y = log(tan(pi / 4 + max_lat_rad / 2))

        y = (max_merc_y - merc_y) / (max_merc_y - min_merc_y) * height
        return int(x), int(y)
    else:
        raise ValueError(f"Unknown projection: {projection}")

def render_shapefile(shp_path, output_path, width=2048, height=1024,
                    bg_color=(0, 0, 0, 255), fill_color=(255, 255, 255, 255),
                    line_color=None, line_width=1, projection='equirectangular'):
    """Render shapefile to PNG image."""
    print(f"Loading shapefile: {shp_path}")
    sf = shapefile.Reader(shp_path)
    shapes = sf.shapes()

    if not shapes:
        print("Error: No shapes found in shapefile")
        return

    print(f"Found {len(shapes)} shape(s)")
    print(f"Shape type: {sf.shapeTypeName}")

    # Get bounds
    min_lon, min_lat, max_lon, max_lat = -180, -90, 180, 90
    print(f"Bounds: ({min_lon:.2f}, {min_lat:.2f}) to ({max_lon:.2f}, {max_lat:.2f})")

    # Create image
    img = Image.new('RGBA', (width, height), bg_color)
    draw = ImageDraw.Draw(img)

    print(f"Rendering {width}x{height} image with {projection} projection...")

    # Draw shapes
    for i, shape in enumerate(shapes):
        if (i + 1) % 100 == 0 or i == len(shapes) - 1:
            print(f"  Processing shape {i + 1}/{len(shapes)}...")

        # Handle different shape types
        shape_type = shape.shapeType

        if shape_type in [shapefile.POLYGON, shapefile.POLYGONZ, shapefile.POLYGONM]:
            # Draw polygons
            parts = list(shape.parts) + [len(shape.points)]
            for j in range(len(parts) - 1):
                start = parts[j]
                end = parts[j + 1]
                points = shape.points[start:end]

                # Project points
                pixel_points = [
                    project_point(lon, lat, min_lon, min_lat, max_lon, max_lat, width, height, projection)
                    for lon, lat in points
                ]

                # Draw filled polygon
                if len(pixel_points) >= 3:
                    draw.polygon(pixel_points, fill=fill_color, outline=line_color, width=line_width)

        elif shape_type in [shapefile.POLYLINE, shapefile.POLYLINEZ, shapefile.POLYLINEM]:
            # Draw polylines
            parts = list(shape.parts) + [len(shape.points)]
            for j in range(len(parts) - 1):
                start = parts[j]
                end = parts[j + 1]
                points = shape.points[start:end]

                # Project points
                pixel_points = [
                    project_point(lon, lat, min_lon, min_lat, max_lon, max_lat, width, height, projection)
                    for lon, lat in points
                ]

                # Draw line
                if len(pixel_points) >= 2:
                    draw.line(pixel_points, fill=line_color or fill_color, width=line_width)

        elif shape_type in [shapefile.POINT, shapefile.POINTZ, shapefile.POINTM]:
            # Draw points
            for lon, lat in shape.points:
                x, y = project_point(lon, lat, min_lon, min_lat, max_lon, max_lat, width, height, projection)
                # Draw small circle
                radius = line_width
                draw.ellipse([x - radius, y - radius, x + radius, y + radius],
                           fill=fill_color, outline=line_color)

        elif shape_type in [shapefile.MULTIPOINT, shapefile.MULTIPOINTZ, shapefile.MULTIPOINTM]:
            # Draw multipoints
            for lon, lat in shape.points:
                x, y = project_point(lon, lat, min_lon, min_lat, max_lon, max_lat, width, height, projection)
                radius = line_width
                draw.ellipse([x - radius, y - radius, x + radius, y + radius],
                           fill=fill_color, outline=line_color)

    # Save image
    img.save(output_path)
    print(f"\n✓ Rendered shapefile to: {output_path}")

def parse_color(color_str):
    """Parse color string to RGBA tuple."""
    if color_str.startswith('#'):
        # Hex color
        color_str = color_str.lstrip('#')
        if len(color_str) == 6:
            return tuple(int(color_str[i:i+2], 16) for i in (0, 2, 4)) + (255,)
        elif len(color_str) == 8:
            return tuple(int(color_str[i:i+2], 16) for i in (0, 2, 4, 6))
    else:
        # Try comma-separated RGB or RGBA
        parts = [int(p.strip()) for p in color_str.split(',')]
        if len(parts) == 3:
            return tuple(parts) + (255,)
        elif len(parts) == 4:
            return tuple(parts)

    raise ValueError(f"Invalid color format: {color_str}")

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description='Render a shapefile to PNG image',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python render_shapefile.py countries.shp world.png
  python render_shapefile.py coastlines.shp coast.png --width 4096 --height 2048
  python render_shapefile.py borders.shp borders.png --bg-color "#000000" --fill-color "#FFFFFF"
  python render_shapefile.py data.shp output.png --projection mercator --line-width 2
        """
    )

    parser.add_argument('input', help='Input shapefile path (.shp)')
    parser.add_argument('output', help='Output PNG path')
    parser.add_argument('--width', type=int, default=8100, help='Output width (default: 8100)')
    parser.add_argument('--height', type=int, default=4050, help='Output height (default: 4050)')
    parser.add_argument('--bg-color', default='#000000', help='Background color (hex or r,g,b,a) (default: #000000)')
    parser.add_argument('--fill-color', default='#FFFFFF', help='Fill color (hex or r,g,b,a) (default: #FFFFFF)')
    parser.add_argument('--line-color', default=None, help='Line/outline color (hex or r,g,b,a) (default: none)')
    parser.add_argument('--line-width', type=int, default=1, help='Line width in pixels (default: 1)')
    parser.add_argument('--projection', choices=['equirectangular', 'mercator'],
                       default='equirectangular', help='Map projection (default: equirectangular)')

    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"Error: Input file '{args.input}' not found")
        sys.exit(1)

    try:
        bg_color = parse_color(args.bg_color)
        fill_color = parse_color(args.fill_color)
        line_color = parse_color(args.line_color) if args.line_color else None

        render_shapefile(
            args.input,
            args.output,
            width=args.width,
            height=args.height,
            bg_color=bg_color,
            fill_color=fill_color,
            line_color=line_color,
            line_width=args.line_width,
            projection=args.projection
        )
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
