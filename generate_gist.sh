#!/bin/sh
path_diff="event_store/src event_store/tests event_store/Cargo.toml"

list=$(git cherry creating_event_store_crate | awk '{print $2}')

SAVEIFS=$IFS   # Save current IFS
IFS=$'\n'      # Change IFS to new line
list=($list) # split to array $names
IFS=$SAVEIFS   # Restore IFS

branch=$(git branch --show-current)
for (( i=0; i<${#list[@]}; i++ ))
do
	sha=${list[$i]}

	file_id=$((i+1))

	files=$(git show --pretty="" --name-only $sha -- $path_diff)
	git co $sha
	RUSTFLAGS=-Awarnings cargo build
	test=$(RUSTFLAGS=-Awarnings cargo test)
	echo "$test" > $1/$(echo $file_id)_test_result.bash
	for file in $files
	do
		content=$(git show $sha:$file)
		if [ $? -eq 0 ]; then
			file_name=$(echo $file | sed "s/event_store//g" |sed "s/\//_/g"|sed "s/^_//g")
			# echo "$content" > $1/$(echo $sha|cut -c -7)_$(echo $file | sed "s/event_store//g" |sed "s/\//_/g"|sed "s/^_//g")
			echo "Generating $sha $1/$(echo $file_id)_$file_name"
			echo "$content" > $1/$(echo $file_id)_$file_name
		fi
	done
done

git co $branch
