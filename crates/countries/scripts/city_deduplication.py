#!/usr/bin/env python3
"""
City Deduplication Script
Finds cities within 50 miles of each other and allows user to select which to keep.
"""

import os
import glob
import math
from typing import List, Dict, Tuple, Set
import re


def parse_ron_file(filename: str) -> Dict:
    """Parse a RON file and extract country data."""
    with open(filename, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract country name
    name_match = re.search(r'name:\s*"([^"]+)"', content)
    code_match = re.search(r'code:\s*"([^"]+)"', content)
    population_match = re.search(r'population:\s*(\d+)', content)
    capital_match = re.search(r'capital:\s*(\d+)', content)
    
    # Extract cities (both active and commented)
    cities = []
    # Pattern matches multiline city format, with optional // comment prefixes
    # This handles both single-line and multi-line format, commented or not
    # The (?://[^\n]*\n\s*)* part matches optional comment lines
    city_pattern = r'((?://\s*)?)\(\s*(?://\s*)?\s*name:\s*"([^"]+)",\s*(?://\s*)?\s*coordinates:\s*\(\s*(?://\s*)?\s*lat:\s*([-\d.]+),\s*(?://\s*)?\s*lon:\s*([-\d.]+),?\s*(?://\s*)?\s*\),?\s*(?://\s*)?\s*\),?'

    for match in re.finditer(city_pattern, content, re.DOTALL):
        is_commented = '//' in match.group(0)  # Check if there's any // in the matched text
        city_data = {
            'name': match.group(2),
            'population': 0,  # Population not stored per city in your format
            'lat': float(match.group(3)),
            'lon': float(match.group(4))
        }
        if is_commented:
            city_data['commented'] = True
        cities.append(city_data)
    
    return {
        'name': name_match.group(1) if name_match else 'Unknown',
        'code': code_match.group(1) if code_match else 'XX',
        'population': int(population_match.group(1)) if population_match else 0,
        'capital': int(capital_match.group(1)) if capital_match else 0,
        'cities': cities
    }


def write_ron_file(filename: str, data: Dict):
    """Write country data back to RON format."""
    ron_content = f"""(
    name: "{data['name']}",
    code: "{data['code']}",
    population: {data['population']},
    capital: {data['capital']},
    cities: [
"""

    for city in data['cities']:
        # Check if city should be commented out
        comment_prefix = "//         " if city.get('commented', False) else "        "
        comment_prefix_inner = "//             " if city.get('commented', False) else "            "

        ron_content += f"""{comment_prefix}(
{comment_prefix_inner}name: "{city['name']}",
{comment_prefix_inner}coordinates: (
{comment_prefix_inner}    lat: {city['lat']},
{comment_prefix_inner}    lon: {city['lon']},
{comment_prefix_inner}),
{comment_prefix}),
"""

    ron_content += """    ],
)
"""

    with open(filename, 'w', encoding='utf-8') as f:
        f.write(ron_content)


def haversine_distance(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """
    Calculate the distance between two points on Earth in miles.
    Uses the Haversine formula.
    """
    # Radius of Earth in miles
    R = 3958.8
    
    # Convert to radians
    lat1_rad = math.radians(lat1)
    lat2_rad = math.radians(lat2)
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    
    # Haversine formula
    a = math.sin(dlat / 2)**2 + math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(dlon / 2)**2
    c = 2 * math.asin(math.sqrt(a))
    
    return R * c


def find_nearby_cities(cities: List[Dict], max_distance: float = 50.0) -> List[List[int]]:
    """
    Find groups of cities that are within max_distance miles of each other.
    Returns a list of city index groups.
    """
    n = len(cities)
    visited = set()
    groups = []
    
    for i in range(n):
        if i in visited:
            continue
        
        # Start a new group with this city
        group = [i]
        visited.add(i)
        
        # Find all cities within max_distance of any city in the current group
        queue = [i]
        while queue:
            current = queue.pop(0)
            current_city = cities[current]
            
            for j in range(n):
                if j in visited:
                    continue
                
                other_city = cities[j]
                distance = haversine_distance(
                    current_city['lat'], current_city['lon'],
                    other_city['lat'], other_city['lon']
                )
                
                if distance <= max_distance:
                    group.append(j)
                    visited.add(j)
                    queue.append(j)
        
        # Only include groups with more than one city
        if len(group) > 1:
            groups.append(sorted(group))
    
    return groups


def display_city_group(cities: List[Dict], indices: List[int], capital_index: int) -> None:
    """Display a group of nearby cities."""
    print("\nThe following cities are within 50 miles of each other:")
    print("-" * 80)

    for idx, city_idx in enumerate(indices, 1):
        city = cities[city_idx]
        is_capital = " [CAPITAL]" if city_idx == capital_index else ""
        print(f"{idx}. {city['name']}{is_capital}")
        print(f"   Coordinates: ({city['lat']:.4f}, {city['lon']:.4f})")


def get_user_choice(num_options: int) -> int:
    """Get user's choice of which city to keep."""
    while True:
        try:
            choice = input(f"\nWhich city do you want to KEEP? (1-{num_options}): ").strip()
            choice_num = int(choice)
            if 1 <= choice_num <= num_options:
                return choice_num - 1  # Convert to 0-indexed
            else:
                print(f"Please enter a number between 1 and {num_options}")
        except ValueError:
            print("Please enter a valid number")
        except KeyboardInterrupt:
            print("\n\nOperation cancelled by user.")
            exit(0)


def process_country_file(filename: str, max_distance: float = 50.0) -> bool:
    """
    Process a single country file to deduplicate nearby cities.
    Returns True if any changes were made.
    """
    print(f"\n{'='*80}")
    print(f"Processing: {filename}")
    print('='*80)
    
    # Parse the file
    data = parse_ron_file(filename)

    # Filter out already commented cities for processing and create index mapping
    active_cities = []
    active_to_original_idx = []  # Maps active city index to original city index

    for orig_idx, city in enumerate(data['cities']):
        if not city.get('commented', False):
            active_cities.append(city)
            active_to_original_idx.append(orig_idx)

    if len(active_cities) <= 1:
        print(f"Only {len(active_cities)} active city in {data['name']}, skipping...")
        return False

    print(f"Country: {data['name']} ({data['code']})")
    print(f"Total cities: {len(data['cities'])} ({len(active_cities)} active)")

    # Find groups of nearby cities (only among active cities)
    groups = find_nearby_cities(active_cities, max_distance)

    if not groups:
        print(f"No cities within {max_distance} miles of each other. Skipping...")
        return False

    print(f"Found {len(groups)} group(s) of nearby cities")

    # Track which cities to remove
    cities_to_remove = set()

    # Process each group
    for group_num, group in enumerate(groups, 1):
        print(f"\n{'*'*80}")
        print(f"Group {group_num} of {len(groups)}")
        print('*'*80)

        # Map active indices back to original indices for display
        original_indices = [active_to_original_idx[i] for i in group]

        display_city_group(data['cities'], original_indices, data['capital'])

        # Get user's choice
        choice_idx = get_user_choice(len(group))
        keep_city_idx = active_to_original_idx[group[choice_idx]]

        print(f"\nKeeping: {data['cities'][keep_city_idx]['name']}")

        # Mark all other cities in group for commenting out
        for active_idx in group:
            original_idx = active_to_original_idx[active_idx]
            if original_idx != keep_city_idx:
                cities_to_remove.add(original_idx)
                print(f"  Commenting out: {data['cities'][original_idx]['name']}")

    # Comment out cities instead of removing them
    if cities_to_remove:
        # Mark cities as commented
        for idx in cities_to_remove:
            data['cities'][idx]['commented'] = True

        # Capital index remains the same since we're not actually removing cities
        if data['capital'] in cities_to_remove:
            print(f"\nWARNING: Capital city was commented out!")

        # Write back to file
        write_ron_file(filename, data)

        # Count active cities (not commented)
        active_cities = sum(1 for city in data['cities'] if not city.get('commented', False))

        print(f"\n{'='*80}")
        print(f"Updated {filename}")
        print(f"Active cities: {active_cities} (commented out {len(cities_to_remove)})")
        print('='*80)
        
        return True
    
    return False


def main():
    """Main function to process all country RON files."""
    print("City Deduplication Tool")
    print("=" * 80)
    print("This script finds cities within 50 miles of each other")
    print("and lets you choose which one to keep.")
    print("=" * 80)
    
    # Find all .country.ron files
    ron_files = glob.glob("*.country.ron")
    
    if not ron_files:
        print("\nNo .country.ron files found in current directory!")
        print("Make sure you run this script in the same directory as the country files.")
        return
    
    print(f"\nFound {len(ron_files)} country file(s)")
    
    # Ask user if they want to process all files
    print("\nOptions:")
    print("1. Process all country files")
    print("2. Process specific country file")
    
    try:
        choice = input("\nEnter your choice (1 or 2): ").strip()
        
        if choice == "2":
            print("\nAvailable country files:")
            for idx, filename in enumerate(sorted(ron_files), 1):
                print(f"{idx}. {filename}")
            
            file_choice = int(input(f"\nSelect file number (1-{len(ron_files)}): ")) - 1
            if 0 <= file_choice < len(ron_files):
                ron_files = [sorted(ron_files)[file_choice]]
            else:
                print("Invalid choice, exiting.")
                return
    except (ValueError, KeyboardInterrupt):
        print("\nExiting...")
        return
    
    # Process each file
    total_processed = 0
    total_modified = 0
    
    for filename in sorted(ron_files):
        try:
            modified = process_country_file(filename)
            total_processed += 1
            if modified:
                total_modified += 1
        except Exception as e:
            print(f"\nError processing {filename}: {e}")
            continue
    
    # Summary
    print(f"\n{'='*80}")
    print("SUMMARY")
    print('='*80)
    print(f"Total files processed: {total_processed}")
    print(f"Total files modified: {total_modified}")
    print(f"Total files unchanged: {total_processed - total_modified}")
    print('='*80)
    print("\nDone!")


if __name__ == "__main__":
    main()