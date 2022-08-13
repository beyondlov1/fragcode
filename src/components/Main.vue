<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { exit } from '@tauri-apps/api/process';
import { register } from '@tauri-apps/api/globalShortcut';


const input = ref('')
const tableData = ref([])
const refInput =ref()
const refTextarea =ref()
const singleTableRef = ref()
const currentRow = ref()
const lastfocus = ref(null)
const showclipboard = ref(false)
const clipboard = ref("")

oninputchange("");


function oninputchange(value){
  console.log(value)
  let keyword = null;
  var command = value.match(/(\w+)(.*)/)
  if(command != null){
    keyword = command[1]
  }else{
    keyword = "";
  }

  if(keyword == "cp"){
    readText().then((value)=> {
        clipboard.value = value
    })
    showclipboard.value = true;
  }else{
    showclipboard.value = false;
  }

  console.log(keyword)
  invoke('list', { name: keyword })
    .then((response) => {
      let r = JSON.parse(response)
      if(command != null && command.length > 2 && command[2].length > 0){
        let args = command[2].trim().split(/\s+/)
        for (let j = 0; j < r.length; j++) {
          var item = r[j];
          for (let i = 0; i < args.length; i++) {
            const arg = args[i];
            item["code"] = replace(item["code"], arg, i+1)
          }
        }
      }
      tableData.value = r;
      setCurrent(tableData.value[0]);
    })
}

function replace(str, arg, argi){
  return str.replaceAll("$"+argi, arg)
}

function onenter(value){
  let command3 = value.match(/(\w+)\s+(\w+)\s+(.*)/)
  if(command3 != null && command3[1] == "add"){
    invoke('add', { abbr: command3[2], code: command3[3] });
    invoke('list', { name: "" })
      .then((response) => {
        tableData.value = JSON.parse(response);
        input.value = "";
      })
    return;
  }
  let command2 = value.match(/(\w+)\s+(\w+)\s?(.*)/)
  if(command2 != null && command2[1] == "cp"){
    invoke('add', { abbr: command2[2], code: clipboard.value });
    invoke('list', { name: "" })
      .then((response) => {
        tableData.value = JSON.parse(response);
        input.value = "";
        showclipboard.value = false;
      })
    return;
  }

  let selectedRow = null;
  if(currentRow.value){
    selectedRow = currentRow.value;
  }else if (tableData.value.length > 0){
    selectedRow  = tableData.value[0]
  }
  if(selectedRow != null){
    writeText(selectedRow["code"]);
    invoke("access", {id:selectedRow["id"]})
    input.value = "";
    oninputchange(input.value);
    invoke("toggle")
  }
}

function reset(){
  input.value = ""
  oninputchange("")
}

function onrowclick(row, column, event){
  console.log(row["code"])
  writeText(row["code"]);
  invoke("access", {id:row["id"]})
  refInput.value.focus();
  refInput.value.select();
  reset()
  invoke("toggle")
}

function remove(row){
  console.log("row"+row.id)
  invoke('remove', { id: row.id});
  oninputchange("");
}

const setCurrent = (row) => {
  if(row){
    singleTableRef.value.setCurrentRow(row)
  }
}
const handleCurrentChange = (val) => {
  currentRow.value = val
}

function focuslast(){
  if(lastfocus.value == null){
    refInput.value.focus();
  }
  if(lastfocus.value != null &&  refTextarea && refTextarea.value != null 
    && lastfocus.value == refTextarea.value && !showclipboard.value){
    refInput.value.focus();
    lastfocus.value = refInput.value;
    return;
  }
  if(lastfocus.value != null){
    lastfocus.value.focus();
  }
}

function oninputfocus(event){
  lastfocus.value = refInput.value
}


function ontextareafocus(event){
  lastfocus.value = refTextarea.value
}


document.onkeydown = function(e) {
  
  if (e.key == "Escape") {
    reset()
    invoke("toggle")
  }
  console.log(e.key)
  if(refInput.value.ref == e.target ){
    if(e.ctrlKey && e.key == "z") {
      input.value = ""
    }
    if(e.key == "Enter"){
      onenter(input.value)
    }
    if(e.key == "Tab"){
      if(refTextarea && refTextarea.value != null && showclipboard.value){
        refTextarea.value.focus();
        refTextarea.value.ref.setSelectionRange(9999,9999);
        return;
      }
    }
    if(e.key == "ArrowDown"){
      let index = 0;
      for (let i = 0; i < tableData.value.length; i++) {
        const item = tableData.value[i];
        if(item["id"] == currentRow.value.id){
          index = i;
          break;
        }
      }
      let next = index+1>=tableData.value.length?tableData.value.length-1:index+1
      singleTableRef.value.setCurrentRow(tableData.value[next])
      return false;
    }
    if(e.key == "ArrowUp"){
      let index = 0;
      for (let i = 0; i < tableData.value.length; i++) {
        const item = tableData.value[i];
        if(item["id"] == currentRow.value.id){
          index = i;
          break;
        }
      }
      let next = index-1<0?0:index-1
      singleTableRef.value.setCurrentRow(tableData.value[next])
      return false;
    }
  }
  if(refTextarea && refTextarea.value && refTextarea.value.ref == e.target ){
    if(e.ctrlKey && e.key == "Enter"){
      onenter(input.value)
    }
  }
}

register('Ctrl+Space', () => {
  invoke("toggle")
});

setInterval(focuslast, 500);

</script>

<template>
  <el-input ref="refInput" v-model="input" @input="oninputchange" @focus="oninputfocus" placeholder="Please input" />
  <el-input v-if="showclipboard" v-model="clipboard" ref="refTextarea" type="textarea" 
   rows="30" @focus="ontextareafocus" placeholder="Clipboard" />
  <el-table v-if="!showclipboard" :data="tableData"  :show-header=false style="width: 100%" @cell-click="onrowclick"
  	  highlight-current-row  ref="singleTableRef" 
      @current-change="handleCurrentChange">
    <el-table-column prop="abbr" label="Abbr." width="100" />
    <el-table-column prop="code" label="Code"/>
    <el-table-column width="100">
      <template v-slot:default="{row}">
        <el-link type="primary" @click.stop="remove(row)" >
          <span>删除</span>
        </el-link>
      </template>
    </el-table-column>
  </el-table>
</template>



