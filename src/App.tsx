import {
  Body1Stronger,
  Button,
  Dropdown,
  FluentProvider,
  Input,
  InputOnChangeData,
  Label,
  Option,
  Spinner,
  webDarkTheme,
  webLightTheme,
} from '@fluentui/react-components'
import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useRef, useState } from 'react'

const HOTKEY_OPTIONS = [
  { key: 'CmdOrCtrl+Alt+V', text: 'Ctrl+Alt+V' },
  { key: 'Alt+V', text: 'Alt+V' },
  { key: 'CmdOrCtrl+1', text: 'Ctrl+1' },
  { key: 'Alt+1', text: 'Alt+1' },
]

export default function App() {
  const [theme, setTheme] = useState(webLightTheme)
  const [errMsg, setErrMsg] = useState('')
  const [stand, setStand] = useState('10')
  const lastStand = useRef('10')
  const [float, setFloat] = useState('5')
  const lastFloat = useRef('5')
  const [counter, setCounter] = useState(-1)
  const [buttonDisabled, setButtonDisabled] = useState(false)
  const [selectedHotkey, setSelectedHotkey] = useState<string[]>(['CmdOrCtrl+Alt+V'])

  const onChange = (
    set: React.Dispatch<React.SetStateAction<string>>,
    _event: React.ChangeEvent<HTMLInputElement>,
    data: InputOnChangeData
  ) => {
    set(data.value)
  }
  const onBlur = (
    current: string,
    set: React.Dispatch<React.SetStateAction<string>>,
    last: React.MutableRefObject<string>
  ) => {
    if (/^[1-9]\d{0,5}$/.test(current)) {
      last.current = current
    } else {
      set(last.current)
    }
  }
  const onClick = () => {
    setButtonDisabled(true)
    setCounter(3)
    const interval = setInterval(() => {
      setCounter((counter) => {
        if (counter == 1) {
          clearInterval(interval)
          ;(async () => {
            try {
              await invoke('paste', {
                stand: parseInt(lastStand.current),
                float: parseInt(lastFloat.current),
              })
              setErrMsg('')
            } catch (e) {
              setErrMsg(e as string)
            }
            setButtonDisabled(false)
            setCounter(-1)
          })()
          return 0
        }
        return counter - 1
      })
    }, 1000)
  }

  const onHotkeyPaste = async () => {
    // 快捷键触发的粘贴，跳过倒计时直接执行
    if (buttonDisabled) return // 如果正在执行中，忽略
    
    setButtonDisabled(true)
    setCounter(0) // 显示执行中状态
    try {
      await invoke('paste_instant', {
        stand: parseInt(lastStand.current),
        float: parseInt(lastFloat.current),
      })
      setErrMsg('')
    } catch (e) {
      setErrMsg(e as string)
    }
    setButtonDisabled(false)
    setCounter(-1)
  }

  const onHotkeyChange = async (_event: any, data: any) => {
    const newHotkey = data.optionValue
    if (newHotkey) {
      setSelectedHotkey([newHotkey])
      try {
        await invoke('set_hotkey', { hotkey: newHotkey })
        // 提示用户需要重启应用以使快捷键生效
        setErrMsg('快捷键已更新，请重启应用以生效')
        setTimeout(() => setErrMsg(''), 3000)
      } catch (e) {
        setErrMsg(e as string)
      }
    }
  }

  useEffect(() => {
    // 设置主题
    const mediaQueryList = window.matchMedia('(prefers-color-scheme: dark)')
    setTheme(mediaQueryList.matches ? webDarkTheme : webLightTheme)
    const themeListener = (event: MediaQueryListEvent) => {
      setTheme(event.matches ? webDarkTheme : webLightTheme)
    }
    mediaQueryList.addEventListener('change', themeListener)

    // 加载初始快捷键配置
    invoke<string>('get_hotkey').then((hotkey) => {
      setSelectedHotkey([hotkey])
    }).catch(console.error)

    // 监听快捷键触发事件
    const unlistenPromise = listen('shortcut-paste', () => {
      console.log('收到快捷键触发事件')
      onHotkeyPaste()
    })

    return () => {
      mediaQueryList.removeEventListener('change', themeListener)
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  return (
    <FluentProvider
      style={{
        width: '100%',
        height: '100%',
      }}
      theme={theme}>
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
        }}>
        <div
          style={{
            display: 'flex',
            padding: 15,
            flexDirection: 'column',
            justifyContent: 'center',
            alignItems: 'center',
          }}>
          {errMsg == '' ? (
            <Body1Stronger>
              单击按钮后, 将在3S后开始, 延迟(ms)范围为[基本延迟,
              基本延迟+浮动值]
            </Body1Stronger>
          ) : (
            <Body1Stronger style={{ color: '#d13438' }}>{errMsg}</Body1Stronger>
          )}

          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-around',
              alignItems: 'flex-end',
              marginBlock: 20,
            }}>
            <div style={{ marginBottom: 10 }}>
              <Label weight="semibold">全局快捷键:</Label>
              <Dropdown
                value={HOTKEY_OPTIONS.find(opt => opt.key === selectedHotkey[0])?.text || 'Ctrl+Alt+V'}
                selectedOptions={selectedHotkey}
                onOptionSelect={onHotkeyChange}
                size="small"
                style={{ marginLeft: 8, minWidth: 130 }}
              >
                {HOTKEY_OPTIONS.map((option) => (
                  <Option key={option.key} value={option.key}>
                    {option.text}
                  </Option>
                ))}
              </Dropdown>
            </div>
            <div style={{ marginBottom: 10 }}>
              <Label weight="semibold">基本延迟:</Label>
              <Input
                value={stand}
                size="small"
                style={{ marginLeft: 8, width: 60 }}
                onChange={onChange.bind(null, setStand)}
                onBlur={onBlur.bind(null, stand, setStand, lastStand)}
              />
            </div>
            <div>
              <Label weight="semibold">浮动值:</Label>
              <Input
                value={float}
                size="small"
                style={{ marginLeft: 8, width: 60 }}
                onChange={onChange.bind(null, setFloat)}
                onBlur={onBlur.bind(null, float, setFloat, lastFloat)}
              />
            </div>
          </div>
          <Button
            appearance="primary"
            disabled={buttonDisabled}
            onClick={onClick}>
            {counter == -1 ? (
              '粘贴'
            ) : counter == 0 ? (
              <Spinner size="tiny" />
            ) : (
              counter
            )}
          </Button>
        </div>
      </div>
    </FluentProvider>
  )
}
