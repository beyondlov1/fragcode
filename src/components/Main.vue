<script setup>
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { exit } from '@tauri-apps/api/process';
import { register } from '@tauri-apps/api/globalShortcut';


const input = ref('')
const hint = ref('')
const currhint = ref("")
const tableData = ref([])
const refInput =ref()
const refTextarea =ref()
const refEdit =ref()
const singleTableRef = ref()
const currentRow = ref()
const lastfocus = ref(null)
const showclipboard = ref(false)
const clipboard = ref("")
const hclipboards = ref([])
const editing = ref(false)


function hintwith(str, ignorelist) {
  for (let i = 0; i < ignorelist.length; i++) {
    const ignorestr = ignorelist[i];
    if(input.value.endsWith(ignorestr)){
      let base = input.value.substring(0, input.value.indexOf(ignorestr));
      hint.value = base + str;
      currhint.value = str;
      return;
    }else{
      break;
    }
  }
  hint.value = input.value + " " + str;
  currhint.value = str;
}

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

  let wildcardkeyword = ''
  for (let i = 0; i < keyword.length; i++) {
    const c = keyword[i];
    wildcardkeyword = wildcardkeyword+"%"+c
  }
  console.log(keyword)
  console.log(wildcardkeyword)
  invoke('list', { name: wildcardkeyword })
    .then((response) => {
      let r = JSON.parse(response)

      //list
      for (let i = 0; i < r.length; i++) {
        const item = r[i];
        item["ocode"] = item["code"]
      }
      if(command != null && command.length > 2 && command[2].trim().length > 0){
        let args = command[2].trim().split(/\s+/)
        for (let j = 0; j < r.length; j++) {
          var item = r[j];
          for (let i = 0; i < args.length; i++) {
            const arg = args[i];
            item["code"] = replace(item["code"], arg, i+1)
          }
        }
      }
      if(command != null && (command.length == 2 || (command.length == 3 && command[2].trim() == ""))){
        console.log("currhint:"+currhint.value)
        let args = [currhint.value]
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
      //list end

      // regexmatch
      if(r.length > 0 && hclipboards.value.length > 1){
        let id = r[0]["id"];
        let candidates = []
        for (let i = 0; i < hclipboards.value.length; i++) {
          const hclip = hclipboards.value[i];
          candidates.push(hclip["code"])
        }
        invoke("rmatch",{id:id,candidates: candidates}).then((resp)=>{
          console.log("rmatch"+resp);
          if(resp[0]>=0){
            // fixme
            currhint.value = resp[1]
            hintwith(currhint.value, [])
          }
        })
      }
      // regexmatch end 
    }).catch((reason)=>{
      console.log("invoke list fail:"+reason)
    })

    if(keyword == "cp"){
      hint.value = ""
      return;
    }
    if(hclipboards.value.length <= 0){
      return;
    }
    if(value == ""){
      hintwith(hclipboards.value[0].code, []);
      return;
    }

    // hint with typing
    let checkedhclip = null;
    for (let i = 0; i < hclipboards.value.length; i++) {
      const hclip = hclipboards.value[i];
      if(hclip.checked){
        checkedhclip = hclip;
        break;
      }
    }

    let ignorelist = []
    if(checkedhclip != null){
      ignorelist.push(checkedhclip.code)  
      hintwith(checkedhclip.code, ignorelist)
    }else{
      hintwith(hclipboards.value[0].code, ignorelist)
    }
    
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
    let arg;
    var command = value.match(/(\w+)(.*)/)
    if(command != null && (command.length == 2 || (command.length == 3 && command[2].trim() == ""))){
      arg = currhint.value
    }else if(command2 != null){
      arg = command2[2]
    }
    console.log("arg:"+arg)
    invoke("train", {id:selectedRow["id"], arg: arg})
    reset()
    invoke("toggle")
  }
}

function reset(){
  editing.value = {on:false}
  input.value = ""
  oninputchange("")
  lastfocus.value = refInput.value
  if(refInput.value)
    refInput.value.focus()
  clearCheckState()
}

function onrowclick(row, column, event){
  console.log(row["code"])
  if("id" in row){
    writeText(row["code"]);
    invoke("access", {id:row["id"]})
  }else{
    writeText(row["code"]);
  }
  refInput.value.focus();
  refInput.value.select();
  reset()
  invoke("toggle")
}

function remove(row){
  console.log("row"+row.id)
  invoke('remove', { id: row.id});
  reset()
}

function showedit(row){
  editing.value = {on:true, id: row.id, code: row.code, ocode: row.ocode, abbr: row.abbr}
}

function edit() {
  if(!editing.value.on){
    return;
  }
  let row = editing.value
  invoke("update", {id: row.id, code: row.ocode, abbr: row.abbr});
  editing.value = {on:false}
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
  if(editing.value.on){
    refEdit.value.focus()
    return;
  }
  if(lastfocus.value == null){
    refInput.value.focus();
    lastfocus.value = refInput.value
  }
  if(lastfocus && lastfocus.value != null 
    && refTextarea && refTextarea.value != null 
    && lastfocus.value == refTextarea.value 
    && !showclipboard.value){
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

function oneditfocus(event){
  lastfocus.value = refEdit.value
}

const onChange = (item) => {
  input.value = input.value + " " + item.code;
  oninputchange(input.value)
}

function stageclipboard() {
  readText().then((x)=>{
    let text = x
    if(x.length > 16){
      text = x.substring(0,16) + ".."
    }
    let tabletext = x
    if(x.length > 300){
      tabletext = x.substring(0,300) + "..."
    }
    let founditem = null;
    let foundi = -1;
    for (let i = 0; i < hclipboards.value.length; i++) {
      const item = hclipboards.value[i];
      if(item["code"] == x){
        founditem = item;
        foundi = i;
        break;
      }
    }
    if(founditem == null){
      hclipboards.value.splice(0, 0, {text:text,tabletext:tabletext,code:x,checked:false});
    }else{
      if(foundi != 0){
        let arr = [];
        let poped;
        while ((poped = hclipboards.value.pop()) != founditem) {
          arr.push(poped);
        }
        for (let i = arr.length - 1; i >= 0; i--) {
          const item = arr[i];
          hclipboards.value.push(item);
        }
        hclipboards.value.splice(0, 0, {text:text,tabletext:tabletext,code:x,checked: false})
      }
    }
    if(hclipboards.value.length > 5){
      hclipboards.value.pop();
    }

    let checkedhclip = null;
    for (let i = 0; i < hclipboards.value.length; i++) {
      const item = hclipboards.value[i];
      if(item.checked){
        checkedhclip = item;
        break;
      }
    }

    let ignorelist = []
    if(checkedhclip != null){
      ignorelist.push(checkedhclip.code)  
      hintwith(checkedhclip.code, ignorelist)
    }else{
      hintwith(hclipboards.value[0].code, ignorelist)
    }
  }).catch((reason)=>{
    console.log("read text error:"+reason);
  })
}


function clearCheckState() {
  for (let i = 0; i < hclipboards.value.length; i++) {
    hclipboards.value[i].checked = false;
  }
}

document.onkeydown = function(e) {

  console.log(e.key)
  if(refInput.value && refInput.value.ref && refInput.value.ref == e.target ){
    if(e.ctrlKey && e.key == "z") {
      input.value = ""
      oninputchange(input.value)
    }
    if(e.ctrlKey && e.key == "e") {
      showedit(currentRow.value)
    }
    if(e.key == "Enter"){
      onenter(input.value)
    }
    if(e.key == "Tab"){
      if(refTextarea && refTextarea.value != null && showclipboard.value){
        refTextarea.value.focus();
        refTextarea.value.ref.setSelectionRange(9999,9999);
        return false;
      }else{
        // checked first
        let checkedhclip = null;
        for (let i = 0; i < hclipboards.value.length; i++) {
          const hclip = hclipboards.value[i];
          if(hclip.checked){
            checkedhclip = hclip;
            break;
          }
        }
        if(checkedhclip == null){
          input.value = input.value +" " + hclipboards.value[0].code;
          hclipboards.value[0].checked = true;
          return false;
        }
        // switch next or fallback to first
        let next = 0;
        let found = null;
        let foundi = -1;
        for (let i = 0; i < hclipboards.value.length; i++) {
          const hclip = hclipboards.value[i];
          if(input.value.endsWith(hclip.code)){
            next = i+1;
            found = hclip;
            foundi = i;
            break;
          }
        }
        if(next >= hclipboards.value.length){
          next = 0;
        }
        if(found){
          input.value = input.value.substring(0, input.value.indexOf(found.code)) + hclipboards.value[next].code;
          clearCheckState()
          found.checked = false;
          hclipboards.value[next].checked = true;
        }else if(hclipboards.value.length > 0){
          input.value = input.value +" " + hclipboards.value[0].code;
          clearCheckState()
          hclipboards.value[next].checked =  true;
        }else{
          // do nothing
        }
        oninputchange(input.value)
        return false;
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
      const tragetElemPostition = document.querySelector(".current-row").offsetTop;
      window.scrollTo(0,tragetElemPostition);
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
      const tragetElemPostition = document.querySelector(".current-row").offsetTop;
      window.scrollTo(0,tragetElemPostition);
      return false;
    }
  }
  if(refTextarea && refTextarea.value && refTextarea.value.ref == e.target ){
    if(e.ctrlKey && e.key == "Enter"){
      onenter(input.value)
      reset()
      return false;
    }
  }

  if(refEdit && refEdit.value && refEdit.value.ref == e.target ){
    if(e.ctrlKey && e.key == "Enter"){
      edit()
      reset()
      return false;
    }
    if(e.key == "Escape"){
      editing.value = {on:false}
      lastfocus.value = refInput.value
      return false;
    }
  }

  if (e.key == "Escape" || (e.ctrlKey && e.key == "c")) {
    invoke("toggle")
    reset()
    return false;
  }
}

register('Ctrl+Space', () => {
  invoke("toggle")
  oninputchange(input.value)
});

onMounted(() => {
  refInput.value.focus()
})

oninputchange("");
setInterval(focuslast, 500);
setInterval(stageclipboard, 1000);

</script>

<template>
  <div v-if="!editing.on">
    <el-input ref="refHint" v-model="hint" input-style="color:#C0C0C0" style="background:transparent;z-index:0;margin-left:0;position:absolute;width:100%;margin-top:0px;color:#FF6633"/>
    <el-input ref="refInput" v-model="input" @input="oninputchange" @focus="oninputfocus" placeholder="" />
  </div>
  <el-input v-if="editing.on" v-model="editing.ocode" ref="refEdit" type="textarea" 
   rows="35" @focus="oneditfocus" placeholder="Edit" />
  <div v-if="!editing.on">
    <div v-if="!showclipboard && input && hclipboards.length" style="text-align:left">
      <el-check-tag v-for="(item,index) in hclipboards" :item="item"
          :index="index"
          :key="item.code" style="margin-right: 8px;"
          :checked="item.checked" 
          @change="onChange(item)">{{item.text}}</el-check-tag>
    </div>
    <el-input v-if="showclipboard" v-model="clipboard" ref="refTextarea" type="textarea" 
    rows="35" @focus="ontextareafocus" placeholder="Clipboard" />
    <el-table v-if="!showclipboard && !input && hclipboards.length" :data="hclipboards"  :show-header=false style="width: 100%" @cell-click="onrowclick"
        highlight-current-row >
      <el-table-column prop="tabletext" label="Code"/>
    </el-table>
    <el-table v-if="!showclipboard" :data="tableData" :show-header=false style="width: 100%" @cell-click="onrowclick"
        highlight-current-row  ref="singleTableRef" 
        @current-change="handleCurrentChange">
      <el-table-column prop="abbr" label="Abbr." width="100" />
      <el-table-column prop="code" label="Code"/>
      <el-table-column width="100">
        <template v-slot:default="{row}">
          <el-link type="primary" @click.stop="showedit(row)" >
            <span>编辑</span>
          </el-link>
          &nbsp;
          <el-link type="primary" @click.stop="remove(row)" >
            <span>删除</span>
          </el-link>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>


<style lang="scss">
.el-textarea__inner,.el-input__inner, .el-input__wrapper {
    background: transparent !important;
}
</style>
